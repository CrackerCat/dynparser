use atom::Atom;
use parser::Parse;
use {parser, Error, error, add_descr_error};


#[derive(Debug)]
pub enum Expression {
    Simple(Atom),
    Or(MultiExpr),
    And(MultiExpr),
    Not(Box<Expression>),
    Repeat(Box<Expression>, NRep, Option<NRep>), //  min max
}


#[derive(Debug)]
pub struct NRep(pub u32);


#[derive(Debug)]
pub struct MultiExpr(pub Vec<Expression>);





impl Parse for Expression {
    fn parse(&self,
             conf: &parser::Config,
             status: parser::Status)
             -> Result<parser::Status, Error> {
        match self {
            &Expression::Simple(ref atom) => atom.parse(conf, status),
            &Expression::Or(MultiExpr(ref exprs)) => parse_or(conf, exprs, status),
            &Expression::And(MultiExpr(ref exprs)) => parse_and(conf, exprs, status),
            &Expression::Not(ref exprs) => parse_negate(conf, exprs, status),
            &Expression::Repeat(ref exprs, ref min, ref max) => {
                parse_repeat(conf, exprs, status, min, max)
            }
        }
    }
}


fn parse_or(conf: &parser::Config,
            exprs: &Vec<Expression>,
            status: parser::Status)
            -> Result<parser::Status, Error> {

    println!("parsing>>>>>>>> or {:?}  {:?}", exprs, status);



    // let mut err = None;//error(&status.pos, "parsing or: ");
    let mut errs = vec![];
    for e in exprs {
        match e.parse(conf, status.clone()) {
            Ok(p) => return Ok(p),
            Err(perr) => errs.push(error(&perr.pos, &format!("{}", perr.descr))),
        }
    }

    let mut error = error(&status.pos, "\nbegin parsing or");
    for e in errs {
        error.descr = format!("{}\n{}", error.descr, e);
    }
    error.descr = format!("{}end parsing or", error.descr);

    Err(error)

    // match err {
    //     Some(err) => Err(err),
    //     None => Err(error(&status.pos, "emtpy or???")),
    // }



    // // let mut deep_error: Option<Error> = None;
    // let mut err = None;//error(&status.pos, "parsing or: ");
    // for e in exprs {
    //     match (e.parse(conf, status.clone()), err) {
    //         (Ok(p), _) => return Ok(p),
    //         (Err(perr), None) => {
    //             err = Some(error(&status.pos, &format!("parsing or:\n  {}", perr)))
    //         }
    //         // deep_error = Some(::deep_error(&deep_error, &error)),
    //         (Err(perr), Some(prev_err)) => {
    //             err = Some(add_descr_error(prev_err, &format!("\n  or {}", perr)))
    //         }
    //     }
    // }

    // match err {
    //     Some(err) => Err(err),
    //     None => Err(error(&status.pos, "emtpy or???")),
    // }
}


fn parse_and(conf: &parser::Config,
             exprs: &Vec<Expression>,
             status: parser::Status)
             -> Result<parser::Status, Error> {
    println!("parsing>>>>>>>>  and {:?}  {:?}", exprs, status);

    let mut parst = status.clone();
    for e in exprs {
        // pending...
        let temp = e.parse(conf, parst.clone());
        println!("temp_________________ {:?}", temp);
        parst = temp?;
        // parst = e.parse(conf, parst.clone())?;
    }
    Ok(parst)
}


fn parse_negate(conf: &parser::Config,
                expr: &Expression,
                status: parser::Status)
                -> Result<parser::Status, Error> {

    match expr.parse(conf, status.clone()) {
        Ok(result) => Err(error(&result.pos, "negation error")),
        Err(_) => Ok(status),
    }
}

fn parse_repeat(conf: &parser::Config,
                expr: &Expression,
                status: parser::Status,
                min: &NRep,
                omax: &Option<NRep>)
                -> Result<parser::Status, Error> {
    println!("parsing...repeat {:?}, {:?}  {:?}", min, omax, status);

    let max_reached = |i| omax.as_ref().map_or(false, |ref m| i + 1 >= m.0);
    let last_ok_or =
        |lok: Option<parser::Status>, ref status| lok.as_ref().unwrap_or(&status).clone();

    let mut opt_lastokst = None;
    for i in 0.. {
        let st = last_ok_or(opt_lastokst.clone(), status.clone());
        // println!("parsing>>>>>repeat {:?}, {:?}  {:?}", min, omax, st);
        let last_result = expr.parse(conf, st);

        opt_lastokst = last_result.clone().ok().or(opt_lastokst);

        match (i >= min.0, max_reached(i), last_result, opt_lastokst.clone()) {
            (false, _, Err(err), _) => {
                return Err(error(&status.pos, &format!("not enougth repetitions. {}", err)))
            }
            (false, _, Ok(_), _) => (),
            (true, true, _, Some(lok)) => {
                println!("******************");
                return Ok(lok);
            }
            (true, true, _, None) => {
                return Err(error(&status.pos, &format!("Inconsistency repeat. {:?}", expr)))
            }
            (true, false, Ok(_), _) => (),
            (true, false, Err(_), Some(lok)) => {
                println!("******************");
                return Ok(lok);
            }
            (true, false, Err(_), None) => {
                println!("******************");
                return Ok(status);
            }
            // {
            //     return Err(error(&status.pos,
            //                      &format!("not enougth repetitions. {:?}", last_result)))
            // }
        }

        // if max_reached(i) || last_result.is_err() {
        //     match (i >= min.0, opt_lastokst.clone()) {
        //         (true, None) => return Ok(status),
        //         (true, Some(st)) => return Ok(st),
        //         (_, _) => //return last_result
        //         {
        //             return Err(error(&status.pos,
        //                              &format!("not enougth repetitions. ")))
        //         }
        //     }
        // }

        // if max_reached(i) {
        //     match (i >= min.0, last_result, opt_lastokst.clone()) {
        //         (true, Err(_), None) => return Ok(status),
        //         (true, Err(_), Some(st)) => return Ok(st),
        //         (true, Ok(st), _) => return Ok(st),
        //         (false, Ok(st), _ ) => return Err(error(&status.pos,
        //                              &format!("Inconsistent repeat rule {:?} ", expr)))
        //         (false, Err(err), _) => return Err(err),


        //         (_, _) => //return last_result
        //         {
        //             return Err(error(&status.pos,
        //                              &format!("not enougth repetitions. ")))
        //         }
        //     }
        // }
    }
    Err(error(&status.pos, "stupid line waitting for #37339"))
}

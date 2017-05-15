use atom::Atom;
use parser::Parse;
use {parser, Error, error, AST};


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
             -> Result<(parser::Status, AST::Node), Error> {
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
            -> Result<(parser::Status, AST::Node), Error> {
    let mut errs = vec![];
    for e in exprs {
        match e.parse(conf, status.clone()) {
            Ok(p) => return Ok(p),
            Err(perr) => errs.push(error(&perr.pos, &perr.descr, conf.text2parse)),
        }
    }

    let mut error = error(&status.pos, "\nbegin parsing or", conf.text2parse);
    let max_deep = errs.iter().fold(0, |acc, e| ::std::cmp::max(acc, e.pos.n2));

    for e in errs {
        if e.pos.n2 == max_deep {
            error.descr = format!("{}\n{}", error.descr, e);
        }
    }
    error.descr = format!("{}end parsing or", error.descr);

    Err(error)
}


fn parse_and(conf: &parser::Config,
             exprs: &Vec<Expression>,
             status: parser::Status)
             -> Result<(parser::Status, AST::Node), Error> {
    let ast = |ast_nodes| {
        AST::Node {
            kind: AST::Kind("and".to_owned()),
            val: AST::Val("".to_owned()),
            nodes: Box::new(ast_nodes),
        }
    };

    let mut parst = status.clone();
    let mut ast_nodes = vec![];
    for e in exprs {
        let (nw_st, ast) = e.parse(conf, parst.clone())?;
        parst = nw_st;
        ast_nodes.push(ast);
    }
    Ok((parst, ast(ast_nodes)))
}


fn parse_negate(conf: &parser::Config,
                expr: &Expression,
                status: parser::Status)
                -> Result<(parser::Status, AST::Node), Error> {

    match expr.parse(conf, status.clone()) {
        Ok(result) => Err(error(&result.0.pos, "negation error", conf.text2parse)),
        Err(_) => Ok((status, AST::from_strs("not", ""))),
    }
}

fn parse_repeat(conf: &parser::Config,
                expr: &Expression,
                status: parser::Status,
                min: &NRep,
                omax: &Option<NRep>)
                -> Result<(parser::Status, AST::Node), Error> {
    let ast = |ast_nodes| {
        AST::Node {
            kind: AST::Kind("repeat".to_owned()),
            val: AST::Val("".to_owned()),
            nodes: ast_nodes,
        }
    };
    let max_reached = |i| omax.as_ref().map_or(false, |ref m| i + 1 >= m.0);
    let last_ok_or =
        |lok: Option<parser::Status>, ref status| lok.as_ref().unwrap_or(&status).clone();

    let mut opt_lastokst = None;
    let mut ast_nodes = Box::new(vec![]);
    for i in 0.. {
        let st = last_ok_or(opt_lastokst.clone(), status.clone());
        let last_result = expr.parse(conf, st);
        let last_result_error = last_result.is_err();

        match last_result {
            Ok((st, ast_node)) => {
                opt_lastokst = Some(st);
                ast_nodes.push(ast_node);
            }
            Err(_) => (),
        }

        match (i >= min.0, max_reached(i), last_result_error, opt_lastokst.clone()) {
            (false, _, true, _) => {
                return Err(error(&status.pos,
                                 &format!("not enougth repetitions."),
                                 conf.text2parse))
            }
            (true, true, _, Some(lok)) => return Ok((lok, ast(ast_nodes))),
            (true, true, _, None) => return Ok((status, ast(ast_nodes))),
            (false, _, false, _) => (),
            (true, false, false, _) => (),
            (true, false, true, Some(lok)) => return Ok((lok, ast(ast_nodes))),
            (true, false, true, None) => return Ok((status, ast(ast_nodes))),
        }
    }
    Err(error(&status.pos,
              "stupid line waitting for #37339",
              conf.text2parse))
}

#![warn(missing_docs)]
//! Module with functions to generate rules from PEG grammar
//!

use ast;
use parse;
use parser;
use std::result;
use Error;

#[cfg(test)]
mod test;

#[derive(Debug)]
pub enum ErrPegAst {
    Peg(Error),
    Ast(String),
}
pub type Result<'a> = result::Result<parser::expression::SetOfRules<'a>, ErrPegAst>;

enum ExprOrRule<'a> {
    Expr(parser::expression::Expression<'a>),
    Rule(parser::expression::SetOfRules<'a>),
}

type ResultExprOrRule<'a> = result::Result<ExprOrRule<'a>, String>;
type ResultExpr<'a> = result::Result<parser::expression::Expression<'a>, String>;

// -------------------------------------------------------------------------------------
//  A P I

/// Given a ```peg``` set of rules on an string, it will generate
/// the set of rules to use in the parser
pub fn rules_from_peg<'a>(peg: &str) -> Result {
    let ast = parse(peg, &rules2parse_peg()).map_err(|e| ErrPegAst::Peg(e))?;

    println!("{:#?}", ast);
    rules_from_ast(&ast)
}

//  A P I
// -------------------------------------------------------------------------------------

fn rules_from_ast<'a>(ast: &ast::Node) -> Result<'a> {
    let ast = ast.compact().prune(&vec!["_"]);
    println!(":::::::  {:?}", ast);

    let result = process_node(&ast).map_err(|e| ErrPegAst::Ast(e))?;

    match result {
        ExprOrRule::Expr(expr) => Ok(rules!("maina" => expr)),
        ExprOrRule::Rule(rule) => Ok(rule),
    }
}

fn process_node<'a>(node: &ast::Node) -> ResultExprOrRule<'a> {
    match node {
        ast::Node::Rule((rname, nodes)) => process_peg_rule(&rname, &nodes),
        _ => Err("ERROR TESTING AST".to_string()),
    }
}

fn process_peg_rule<'a>(rname: &str, nodes: &[ast::Node]) -> ResultExprOrRule<'a> {
    match rname {
        "main" => passthrow(&nodes),
        "grammar" => passthrow(&nodes),
        "rule" => process_rule(&nodes),
        "expr" => passthrow(&nodes),
        "or" => passthrow(&nodes),
        "and" => passthrow(&nodes),
        "rep_or_neg" => passthrow(&nodes),
        "atom_or_par" => passthrow(&nodes),
        "atom" => Ok(ExprOrRule::Expr(process_atom(&nodes)?)),
        _ => Err(format!("unknown peg rule {}", rname)),
    }.or_else(|e| Err(format!("processing {} > {:#?}", rname, e)))
}

fn get_symbol_value<'a>(node: &ast::Node) -> ResultExprOrRule<'a> {
    let (nname, nodes) = ast::get_nodename_and_nodes(node)?;
    match nname {
        "symbol" => ast::get_nodes_unique_val(nodes),
        _ => Err(ast::error("expected symbol", None)),
    }
}

fn process_rule<'a>(nodes: &[ast::Node]) -> ResultExprOrRule<'a> {
    //  rule            =   symbol  _  "="  _   expr  (_ / eof)

    Ok(ExprOrRule::Rule(rules!("main" => lit!("testing"))))

    // let symbol_name = get_symbol_value(&nodes[0])?;
    // println!("____ {:?}", symbol_name);

    // match get_node_unique_val(&nodes[1])? == "=" {
    //     false => Err(format!("Expected '=' reading rule {:?}", nodes))?,
    //     _ => (),
    // };

    //Err("error processing rule".to_string())
}

fn passthrow<'a>(nodes: &[ast::Node]) -> ResultExprOrRule<'a> {
    match nodes {
        [node] => process_node(node),
        _ => Err(format!(
            "passthrow can have only one child node {:?}",
            nodes
        )),
    }
}

fn process_atom<'a, 'b>(nodes: &'b [ast::Node]) -> ResultExpr<'a> {
    let get_atom_child_node = |nodes: &'b [ast::Node]| match nodes {
        &[ref node] => Ok(node),
        _ => Err(format!("an atom can have only one child {:?}", &nodes)),
    };

    let get_atom_rule_info = |&node| match node {
        &ast::Node::Rule((ref name, ref nodes)) => Ok((name, nodes)),
        _ => Err(format!("incorrect atom info in ast {:?}", &nodes)),
    };

    let atom_node = get_atom_child_node(nodes)?;
    let (rname, nodes) = get_atom_rule_info(&atom_node)?;

    match (&rname as &str, nodes) {
        ("literal", nodes) => atom_literal_from_nodes(&nodes),
        // ("symbol", nodes) => atom_symbol_from_nodes(&nodes),
        (at, _) => Err(format!("not registered atom type {}", at)),
    }
}

fn atom_literal_from_nodes<'a, 'b>(nodes: &'b [ast::Node]) -> ResultExpr<'a> {
    //  literal =   "\""  (!"\"" .)*  "\""

    let check_quote = |n: &ast::Node| match n {
        ast::Node::Val(v) => {
            if v == "\"" {
                Ok(())
            } else {
                Err(format!("Expected quote arround literal string, got {}", v))
            }
        }
        _ => Err(format!(
            "Expected ast::Node::Val arround literal string, got {:?}",
            n
        )),
    };

    let remove_quotes_arround = |nodes: &'b [ast::Node]| -> result::Result<&[ast::Node], String> {
        let msg_inv_nodes_size = || {
            format!(
                "Invalid ast for literal. Minimum nodes size 3 '{:?}''",
                &nodes
            )
        };
        let (f, nodes) = nodes.split_first().ok_or(msg_inv_nodes_size())?;
        let (l, nodes) = nodes.split_last().ok_or(msg_inv_nodes_size())?;
        let (_, _) = (check_quote(f)?, check_quote(l)?);
        Ok(nodes)
    };

    let concat_str_nodes2string = |nodes: &[ast::Node]| {
        nodes
            .iter()
            .try_fold("".to_string(), |acc, n: &ast::Node| match n {
                ast::Node::Val(v) => Ok(format!("{}{}", acc, v)),
                _ => Err(format!("Expected ast::Node::Val {:?}", &n)),
            })
    };

    let removed_quotes = remove_quotes_arround(nodes)?;
    let slit = concat_str_nodes2string(removed_quotes)?;

    Ok(lit!(slit))
}

fn atom_symbol_from_nodes<'a, 'b>(nodes: &'b [ast::Node]) -> result::Result<String, String> {
    //  symbol          =   [a-zA-Z0-9_']+

    Ok("symbol".to_string())
}

// fn atom_match_from_nodes<'a>(nodes: &'a [ast::Node]) -> ResultExpr<'a> {
//     //  match   =   "["  ((.  "-"  .)  /  (.))+   "]"

//     // ex
//     // Val("["),
//     // Val("a"),
//     // Val("b"),
//     // Val("A"),
//     // Val("-"),
//     // Val("Z"),
//     // Val("]")

// }

fn atom_dot_from_nodes<'a>(nodes: &'a [ast::Node]) -> ResultExpr<'a> {
    //  dot     =   "."

    let get_dot = |val| match val {
        "." => Ok(dot!()),
        _ => Err(format!(
            "Error extracting dot from '{}'\nExpetected '.'",
            val
        )),
    };

    match nodes[..] {
        [ast::Node::Val(ref val)] => get_dot(&val),
        _ => Err("Error extracting literal expected 1 child val nodes".to_string()),
    }
}

fn atom_ref_rule_from_nodes<'a>(nodes: &'a [ast::Node]) -> ResultExpr<'a> {
    //  symbol  =   [a-zA-Z0-9_]+

    fn concat_val_lit_nodes<'a>(
        nodes: &'a [ast::Node],
        acc: String,
    ) -> result::Result<String, String> {
        let concat_node = |n: &_, acc: String| match n {
            ast::Node::Val(ref v) => Ok(format!("{}{}", acc, v)),
            _ => Err("Expected ast::Node::Val(String)"),
        };

        let r_name = match nodes.len() {
            0 => acc,
            _ => concat_val_lit_nodes(&nodes[1..], concat_node(&nodes[0], acc)?)?,
        };
        Ok(r_name)
    };

    let r_name = concat_val_lit_nodes(nodes, "".to_string())?;
    Ok(rule!(r_name))
}

//  ------------------------------------------------------------------------
//  ------------------------------------------------------------------------
//
//  this is the first version of code to parse the peg grammar
//  it was, obviously written by hand
fn rules2parse_peg<'a>() -> parser::expression::SetOfRules<'a> {
    rules!(

        "main"      =>       rule!("grammar"),

        "grammar"   =>       rep!(rule!("rule"), 1),

        "rule"      =>       and!(
                                 rule!("_"), rule!("symbol") ,
                                 rule!("_"), lit! ("="),
                                 rule!("_"), rule!("expr"),
                                             or!(
                                                 rule!("_"),
                                                 rule!("eof")
                                             ),
                                 rule!("_")                                                
                             ),

        "expr"      =>      rule!("or"),

        "or"        =>      and!(
                                rule!("and"),
                                rep!(
                                    and!(
                                        rule!("_"), lit!("/"),
                                        rule!("_"), rule!("or")
                                    ),
                                    0
                                )
                            ),

        "and"       =>     and!(
                                rule!("rep_or_neg"),
                                rep!(
                                    and!(
                                        lit!(" "),  rule!("_"), rule!("and")
                                    ),
                                    0
                                )
                            ),

        "rep_or_neg" =>     or!(
                                and!(
                                    rule!("atom_or_par"),
                                    rep!(
                                        or!(
                                            lit!("*"),
                                            lit!("+"),
                                            lit!("?")
                                        )
                                        , 0, 1
                                    )
                                ),
                                and!(
                                    lit!("!"),
                                    rule!("atom_or_par")
                                )
                            ),

        "atom_or_par" =>    or!(
                                rule!("atom"),
                                rule!("parenth")
                            ),

        "parenth"       =>  and!(
                                lit!("("),
                                rule!("_"),
                                rule!("expr"),
                                rule!("_"),
                                lit!(")")
                            ),

        "atom"          =>  or!(
                                rule!("literal"),
                                rule!("match"),
                                rule!("dot"),
                                rule!("symbol")
                            ),

        "literal"       =>  and!(
                                lit!(r#"""#),
                                rep!(
                                    and!(
                                        not!(
                                            lit!(r#"""#)
                                        ),
                                        dot!()
                                    )
                                    , 0
                                ),
                                lit!(r#"""#)
                            ),

        "match"         =>  and!(
                                lit!("["),
                                or!(
                                    and!(dot!(), lit!("-"), dot!()),
                                    rep!(
                                        and!(not!(lit!("]")), dot!())
                                        ,1
                                    )
                                ),
                                lit!("]")
                            ),
        
        "dot"           =>  lit!("."),

        "symbol"        =>  rep!(
                                ematch!(    chlist "_'",
                                         from 'a', to 'z',
                                         from 'A', to 'Z',
                                         from '0', to '9'
                                ),
                                1
                            ),

        "_"             =>  rep!(   or!(
                                        lit!(" "),
                                        rule!("eol"),
                                        rule!("comment")
                                    )
                                    , 0
                            ),

        "eol"          =>   or!(
                                    lit!("\r\n"),
                                    lit!("\n"),
                                    lit!("\r")
                                ),
        "comment"       =>  or!(
                                and!(
                                    lit!("//"),
                                    rep!(
                                        and!(
                                            not!(rule!("eol")),
                                            dot!()
                                        )
                                        , 0
                                    ),
                                    rule!("eol")
                                ),
                                and!(
                                    lit!("/*"),
                                    rep!(
                                        and!(
                                            not!(lit!("*/")),
                                            dot!()
                                        )
                                        , 0
                                    ),
                                    lit!("*/")
                                )
                        )
    )
}

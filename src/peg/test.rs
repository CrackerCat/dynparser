//-----------------------------------------------------------------------
//
//  mod peg  TEST
//
//-----------------------------------------------------------------------
use parse;
use peg;

#[test]
fn validate_peg1() {
    let peg = r#"
    
    main    = "aaa" "bb" c*  / ((d f)+ / es)
    c       = c'
    c'      = c''
    c''     = "c"
    d       = "d"
    f       = "f"
    es       = "e"+

    "#;

    assert!(parse(peg, &peg::rules::parse_peg()).is_ok());
}

#[test]
fn validate_peg2() {
    let peg = r#"
    
    main    =   "aaa" 
            /   "bb" c*  
            / ((d f)+ / es)

    c       = c'
    c'      = c''
    c''     = "c"
    d       = "d"
    f       = "f"
    es       = "e"+

    "#;

    assert!(parse(peg, &peg::rules::parse_peg()).is_ok());
}

#[test]
fn invalid_peg1() {
    let peg = r#"
    
    main    = "aaa" "bb" c *  / ((d f)+ / es)
    c       = c'
    c'      = c''
    c''     = "c"
    d       = "d"
    f       = "f"
    es       = "e"+

    "#;

    assert!(parse(peg, &peg::rules::parse_peg()).is_err());
}

#[test]
fn invalid_peg2() {
    let peg = r#"
    
    main    =   "aaa" 
            /   "bb" c*  
            / ((d f)+ / es
            
    c       = c'
    c'      = c''
    c''     = "c"
    d       = "d"
    f       = "f"
    es       = "e"+

    "#;

    assert!(parse(peg, &peg::rules::parse_peg()).is_err());
}

#[test]
fn validate_peg3() {
    let peg = r#"
    
    main    = "aaa "bb" c*  / ((d f)+ / es)
    c       = c'
    c'      = c''
    c''     = "c"
    d       = "d"
    f       = "f"
    es       = "e"+

    "#;

    assert!(parse(peg, &peg::rules::parse_peg()).is_err());
}

#[test]
fn validate_peg4() {
    let peg = r#"
    
    ma in    = "aaa" "bb" c*  / ((d f)+ / es)
    c       = c'
    c'      = c''
    c''     = "c"
    d       = "d"
    f       = "f"
    es       = "e"+

    "#;

    assert!(parse(peg, &peg::rules::parse_peg()).is_err());
}

#[test]
fn validate_peg5() {
    let peg = r#"
    
    main    = "aaa" "bb" c*  / ((d f)+* / es)
    c       = c'
    c'      = c''
    c''     = "c"
    d       = "d"
    f       = "f"
    es       = "e"+

    "#;

    assert!(parse(peg, &peg::rules::parse_peg()).is_err());
}

#[test]
fn parse_literal() {
    let peg = r#"

    main    = "hello"

    "#;

    let rules = peg::rules_from_peg(peg).unwrap();

    assert!(parse("hello", &rules).is_ok());
}

#[test]
fn parse_and_literal() {
    let peg = r#"

    main    = "hello"   " "   "world"

    "#;

    let rules = peg::rules_from_peg(peg).unwrap();

    assert!(parse("hello world", &rules).is_ok());
}

#[test]
fn parse_or_literal() {
    let peg = r#"

    main    = "hello"   /   "hola"

    "#;

    let rules = peg::rules_from_peg(peg).unwrap();

    assert!(parse("hello", &rules).is_ok());
    assert!(parse("hola", &rules).is_ok());
    assert!(parse("bye", &rules).is_err());
}

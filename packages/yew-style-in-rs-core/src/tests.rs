use super::ast::*;
use super::transpiler::*;

#[test]
fn test_runtime_parse_1() {
    let runtime_css = RuntimeCss::parse(
        r#"
            border: solid 1px black;
        "#,
    )
    .expect("Parse Error!");

    assert!(matches!(runtime_css.0[0], Declaration::Property(_)));

    if let Declaration::Property(prop) = &runtime_css.0[0] {
        assert_eq!(prop.property, "border");
        assert_eq!(prop.value, "solid 1px black");
    }
}

#[test]
fn test_runtime_parse_2() {
    let runtime_css = RuntimeCss::parse(
        r#"
            @media (orientation: landscape) {
                grid-auto-flow: column
            }
        "#,
    )
    .expect("Parse Error!");

    assert!(matches!(runtime_css.0[0], Declaration::AtRule(_)));

    if let Declaration::AtRule(at_rule) = &runtime_css.0[0] {
        assert_eq!(at_rule.rule_name, "media");
        assert_eq!(at_rule.rule_value, "(orientation: landscape)");
        assert!(at_rule.block.is_some());
        assert_eq!(at_rule.block.as_ref().unwrap().len(), 1);

        assert!(matches!(
            at_rule.block.as_ref().unwrap()[0],
            Declaration::Property(_)
        ));

        if let Declaration::Property(prop) = &runtime_css.0[0] {
            assert_eq!(prop.property, "grid-auto-flow");
            assert_eq!(prop.value, "column");
        }
    }
}

#[test]
fn test_runtime_parse_3() {
    let runtime_css = RuntimeCss::parse(
        r#"
            & .classA, & > .classB {
                border: solid 1px black;
                background: url("./background.png");
            }
        "#,
    )
    .expect("Parse Error!");

    assert!(matches!(runtime_css.0[0], Declaration::QualifiedRule(_)));

    if let Declaration::QualifiedRule(rule) = &runtime_css.0[0] {
        assert_eq!(rule.selectors.0.len(), 2);
        assert_eq!(rule.selectors.0[0], "& .classA".to_string());
        assert_eq!(rule.selectors.0[1], "& > .classB".to_string());

        assert_eq!(rule.block.len(), 2);
        assert!(matches!(rule.block[0], Declaration::Property(_)));
        assert!(matches!(rule.block[1], Declaration::Property(_)));

        if let Declaration::Property(prop) = &rule.block[0] {
            assert_eq!(prop.property, "border");
            assert_eq!(prop.value, "solid 1px black");
        }

        if let Declaration::Property(prop) = &rule.block[1] {
            assert_eq!(prop.property, "background");
            assert_eq!(prop.value, r#"url("./background.png")"#);
        }
    }
}

#[test]
fn test_runtime_parse_4() {
    let runtime_css = RuntimeCss::parse(
        r#"
            border: solid 1px black;
            @media (orientation: landscape) {
                border: solid 10px cyan;
            }
            border: solid 1px black;
            & .classA {
                background: url(./background.png);
                & > .classB {
                    background: rgb(255, 128, 0)
                }
            }
        "#,
    )
    .expect("Parse Error!");

    assert_eq!(runtime_css.0.len(), 4);

    assert!(matches!(runtime_css.0[0], Declaration::Property(_)));
    assert!(matches!(runtime_css.0[1], Declaration::AtRule(_)));
    assert!(matches!(runtime_css.0[2], Declaration::Property(_)));
    assert!(matches!(runtime_css.0[3], Declaration::QualifiedRule(_)));

    if let Declaration::Property(prop) = &runtime_css.0[0] {
        assert_eq!(prop.property, "border");
        assert_eq!(prop.value, "solid 1px black");
    }

    if let Declaration::AtRule(at_rule) = &runtime_css.0[1] {
        assert_eq!(at_rule.rule_name, "media");
        assert_eq!(at_rule.rule_value, "(orientation: landscape)");
        assert!(at_rule.block.is_some());
        assert_eq!(at_rule.block.as_ref().unwrap().len(), 1);

        assert!(matches!(
            at_rule.block.as_ref().unwrap()[0],
            Declaration::Property(_)
        ));

        if let Declaration::Property(prop) = &at_rule.block.as_ref().unwrap()[0] {
            assert_eq!(prop.property, "border");
            assert_eq!(prop.value, "solid 10px cyan");
        }
    }

    if let Declaration::Property(prop) = &runtime_css.0[2] {
        assert_eq!(prop.property, "border");
        assert_eq!(prop.value, "solid 1px black");
    }

    if let Declaration::QualifiedRule(rule) = &runtime_css.0[3] {
        assert_eq!(rule.selectors.0.len(), 1);
        assert_eq!(rule.selectors.0[0], "& .classA".to_string());

        assert_eq!(rule.block.len(), 2);

        assert!(matches!(rule.block[0], Declaration::Property(_)));
        assert!(matches!(rule.block[1], Declaration::QualifiedRule(_)));

        if let Declaration::Property(prop) = &rule.block[0] {
            assert_eq!(prop.property, "background");
            assert_eq!(prop.value, "url(./background.png)");
        }

        if let Declaration::QualifiedRule(rule) = &rule.block[1] {
            assert_eq!(rule.selectors.0.len(), 1);
            assert_eq!(rule.selectors.0[0], "& > .classB".to_string());

            assert_eq!(rule.block.len(), 1);
            assert!(matches!(rule.block[0], Declaration::Property(_)));

            if let Declaration::Property(prop) = &rule.block[0] {
                assert_eq!(prop.property, "background");
                assert_eq!(prop.value, "rgb(255, 128, 0)");
            }
        }
    }
}

#[test]
fn test_runtime_parse_5() {
    let runtime_css = RuntimeCss::parse(
        r#"
            border solid 1px black;
        "#,
    )
    .expect_err("Expected parse error!")
    .0;

    assert_eq!(runtime_css.0.len(), 0);
}

#[test]
fn test_runtime_parse_6() {
    let runtime_css = RuntimeCss::parse(
        r#"
            &:is(.bar, &.baz) {
                border: solid 1px black;
                background: url("./background.png");
            }
        "#,
    )
    .expect("Parse Error!");

    assert!(matches!(runtime_css.0[0], Declaration::QualifiedRule(_)));

    if let Declaration::QualifiedRule(rule) = &runtime_css.0[0] {
        assert_eq!(rule.selectors.0.len(), 1);
        assert_eq!(rule.selectors.0[0], "&:is(.bar, &.baz)".to_string());

        assert_eq!(rule.block.len(), 2);
        assert!(matches!(rule.block[0], Declaration::Property(_)));
        assert!(matches!(rule.block[1], Declaration::Property(_)));

        if let Declaration::Property(prop) = &rule.block[0] {
            assert_eq!(prop.property, "border");
            assert_eq!(prop.value, "solid 1px black");
        }

        if let Declaration::Property(prop) = &rule.block[1] {
            assert_eq!(prop.property, "background");
            assert_eq!(prop.value, r#"url("./background.png")"#);
        }
    }
}

#[test]
fn test_runtime_parse_7() {
    let runtime_css = RuntimeCss::parse(
        r#"
            &:is(.bar, &.baz) {
                border: solid 1px black;
                background url("./background.png");
            }
        "#,
    )
    .expect_err("Expected parse error")
    .0;

    assert!(matches!(runtime_css.0[0], Declaration::QualifiedRule(_)));

    if let Declaration::QualifiedRule(rule) = &runtime_css.0[0] {
        assert_eq!(rule.selectors.0.len(), 1);
        assert_eq!(rule.selectors.0[0], "&:is(.bar, &.baz)".to_string());

        assert_eq!(rule.block.len(), 1);
        assert!(matches!(rule.block[0], Declaration::Property(_)));

        if let Declaration::Property(prop) = &rule.block[0] {
            assert_eq!(prop.property, "border");
            assert_eq!(prop.value, "solid 1px black");
        }
    }
}

#[test]
fn test_transpile_1() {
    // .foo {
    //   color: blue;
    //   & > .bar { color: red; }
    // }
    let runtime_css = RuntimeCss(vec![
        Declaration::Property(Property {
            property: "color".into(),
            value: "blue".into(),
        }),
        Declaration::QualifiedRule(QualifiedRule {
            selectors: Selectors(vec!["& > .bar".into()]),
            block: vec![Declaration::Property(Property {
                property: "color".into(),
                value: "red".into(),
            })],
        }),
    ]);

    let transpiled_css = TranspiledCss::transpile(&[".foo"], runtime_css);

    // .foo { color: blue; }
    // .foo > .bar { color: red; }
    let expected_css = TranspiledCss(vec![
        Rule::QualifiedRule(QualifiedRule {
            selectors: Selectors(vec![".foo".into()]),
            block: vec![Declaration::Property(Property {
                property: "color".into(),
                value: "blue".into(),
            })],
        }),
        Rule::QualifiedRule(QualifiedRule {
            selectors: Selectors(vec![".foo > .bar".into()]),
            block: vec![Declaration::Property(Property {
                property: "color".into(),
                value: "red".into(),
            })],
        }),
    ]);

    assert_eq!(transpiled_css, expected_css);
}

#[test]
fn test_transpile_2() {
    //.foo {
    //   color: blue;
    //   &.bar { color: red; }
    // }
    let runtime_css = RuntimeCss(vec![
        Declaration::Property(Property {
            property: "color".into(),
            value: "blue".into(),
        }),
        Declaration::QualifiedRule(QualifiedRule {
            selectors: Selectors(vec!["&.bar".into()]),
            block: vec![Declaration::Property(Property {
                property: "color".into(),
                value: "red".into(),
            })],
        }),
    ]);

    let transpiled_css = TranspiledCss::transpile(&[".foo"], runtime_css);

    // .foo { color: blue; }
    // .foo.bar { color: red; }
    let expected_css = TranspiledCss(vec![
        Rule::QualifiedRule(QualifiedRule {
            selectors: Selectors(vec![".foo".into()]),
            block: vec![Declaration::Property(Property {
                property: "color".into(),
                value: "blue".into(),
            })],
        }),
        Rule::QualifiedRule(QualifiedRule {
            selectors: Selectors(vec![".foo.bar".into()]),
            block: vec![Declaration::Property(Property {
                property: "color".into(),
                value: "red".into(),
            })],
        }),
    ]);

    assert_eq!(transpiled_css, expected_css);
}

#[test]
fn test_transpile_3() {
    // .foo, .bar {
    //   color: blue;
    //   & + .baz, &.qux { color: red; }
    // }
    let runtime_css = RuntimeCss(vec![
        Declaration::Property(Property {
            property: "color".into(),
            value: "blue".into(),
        }),
        Declaration::QualifiedRule(QualifiedRule {
            selectors: Selectors(vec!["& + .baz".into(), "&.qux".into()]),
            block: vec![Declaration::Property(Property {
                property: "color".into(),
                value: "red".into(),
            })],
        }),
    ]);

    let transpiled_css = TranspiledCss::transpile(&[".foo", ".bar"], runtime_css);

    // .foo, .bar { color: blue; }
    // :is(.foo, .bar) + .baz,
    // :is(.foo, .bar).qux { color: red; }
    let expected_css = TranspiledCss(vec![
        Rule::QualifiedRule(QualifiedRule {
            selectors: Selectors(vec![".foo".into(), ".bar".into()]),
            block: vec![Declaration::Property(Property {
                property: "color".into(),
                value: "blue".into(),
            })],
        }),
        Rule::QualifiedRule(QualifiedRule {
            selectors: Selectors(vec![
                ":is(.foo, .bar) + .baz".into(),
                ":is(.foo, .bar).qux".into(),
            ]),
            block: vec![Declaration::Property(Property {
                property: "color".into(),
                value: "red".into(),
            })],
        }),
    ]);

    assert_eq!(transpiled_css, expected_css);
}

#[test]
fn test_transpile_4() {
    // .foo {
    //   color: blue;
    //   & .bar & .baz & .qux { color: red; }
    // }
    let runtime_css = RuntimeCss(vec![
        Declaration::Property(Property {
            property: "color".into(),
            value: "blue".into(),
        }),
        Declaration::QualifiedRule(QualifiedRule {
            selectors: Selectors(vec!["& .bar & .baz & .qux".into()]),
            block: vec![Declaration::Property(Property {
                property: "color".into(),
                value: "red".into(),
            })],
        }),
    ]);

    let transpiled_css = TranspiledCss::transpile(&[".foo"], runtime_css);

    // .foo { color: blue; }
    // .foo .bar .foo .baz .foo .qux { color: red; }
    let expected_css = TranspiledCss(vec![
        Rule::QualifiedRule(QualifiedRule {
            selectors: Selectors(vec![".foo".into()]),
            block: vec![Declaration::Property(Property {
                property: "color".into(),
                value: "blue".into(),
            })],
        }),
        Rule::QualifiedRule(QualifiedRule {
            selectors: Selectors(vec![".foo .bar .foo .baz .foo .qux".into()]),
            block: vec![Declaration::Property(Property {
                property: "color".into(),
                value: "red".into(),
            })],
        }),
    ]);

    assert_eq!(transpiled_css, expected_css);
}

#[test]
fn test_transpile_5() {
    // .foo {
    //   color: blue;
    //   & { padding: 2ch; }
    // }
    let runtime_css = RuntimeCss(vec![
        Declaration::Property(Property {
            property: "color".into(),
            value: "blue".into(),
        }),
        Declaration::QualifiedRule(QualifiedRule {
            selectors: Selectors(vec!["&".into()]),
            block: vec![Declaration::Property(Property {
                property: "padding".into(),
                value: "2ch".into(),
            })],
        }),
    ]);

    let transpiled_css = TranspiledCss::transpile(&[".foo"], runtime_css);

    // .foo {
    //   color: blue;
    //   padding: 2ch;
    // }
    let expected_css = TranspiledCss(vec![Rule::QualifiedRule(QualifiedRule {
        selectors: Selectors(vec![".foo".into()]),
        block: vec![
            Declaration::Property(Property {
                property: "color".into(),
                value: "blue".into(),
            }),
            Declaration::Property(Property {
                property: "padding".into(),
                value: "2ch".into(),
            }),
        ],
    })]);

    assert_eq!(transpiled_css, expected_css);
}

#[test]
fn test_transpile_6() {
    // .foo {
    //   color: blue;
    //   && { padding: 2ch; }
    // }
    let runtime_css = RuntimeCss(vec![
        Declaration::Property(Property {
            property: "color".into(),
            value: "blue".into(),
        }),
        Declaration::QualifiedRule(QualifiedRule {
            selectors: Selectors(vec!["&&".into()]),
            block: vec![Declaration::Property(Property {
                property: "padding".into(),
                value: "2ch".into(),
            })],
        }),
    ]);

    let transpiled_css = TranspiledCss::transpile(&[".foo"], runtime_css);

    // .foo { color: blue; }
    // .foo.foo { padding: 2ch; }
    let expected_css = TranspiledCss(vec![
        Rule::QualifiedRule(QualifiedRule {
            selectors: Selectors(vec![".foo".into()]),
            block: vec![Declaration::Property(Property {
                property: "color".into(),
                value: "blue".into(),
            })],
        }),
        Rule::QualifiedRule(QualifiedRule {
            selectors: Selectors(vec![".foo.foo".into()]),
            block: vec![Declaration::Property(Property {
                property: "padding".into(),
                value: "2ch".into(),
            })],
        }),
    ]);

    assert_eq!(transpiled_css, expected_css);
}

#[test]
fn test_transpile_7() {
    // .error, #404 {
    //   &:hover > .baz { color: red; }
    // }
    let runtime_css = RuntimeCss(vec![Declaration::QualifiedRule(QualifiedRule {
        selectors: Selectors(vec!["&:hover > .baz".into()]),
        block: vec![Declaration::Property(Property {
            property: "color".into(),
            value: "red".into(),
        })],
    })]);

    let transpiled_css = TranspiledCss::transpile(&[".error", "#404"], runtime_css);

    // :is(.error, #404):hover > .baz { color: red; }
    let expected_css = TranspiledCss(vec![Rule::QualifiedRule(QualifiedRule {
        selectors: Selectors(vec![":is(.error, #404):hover > .baz".into()]),
        block: vec![Declaration::Property(Property {
            property: "color".into(),
            value: "red".into(),
        })],
    })]);

    assert_eq!(transpiled_css, expected_css);
}

#[test]
fn test_transpile_8() {
    // .foo {
    //   &:is(.bar, &.baz) { color: red; }
    // }
    let runtime_css = RuntimeCss(vec![Declaration::QualifiedRule(QualifiedRule {
        selectors: Selectors(vec!["&:is(.bar, &.baz)".into()]),
        block: vec![Declaration::Property(Property {
            property: "color".into(),
            value: "red".into(),
        })],
    })]);

    let transpiled_css = TranspiledCss::transpile(&[".foo"], runtime_css);

    // .foo:is(.bar, .foo.baz) { color: red; }
    let expected_css = TranspiledCss(vec![Rule::QualifiedRule(QualifiedRule {
        selectors: Selectors(vec![".foo:is(.bar, .foo.baz)".into()]),
        block: vec![Declaration::Property(Property {
            property: "color".into(),
            value: "red".into(),
        })],
    })]);

    assert_eq!(transpiled_css, expected_css);
}

#[test]
fn test_transpile_9() {
    // figure {
    //   margin: 0;
    //   & > figcaption {
    //     background: hsl(0 0% 0% / 50%);
    //     & > p {
    //       font-size: .9rem;
    //     }
    //   }
    // }
    let runtime_css = RuntimeCss(vec![
        Declaration::Property(Property {
            property: "margin".into(),
            value: "0".into(),
        }),
        Declaration::QualifiedRule(QualifiedRule {
            selectors: Selectors(vec!["& > figcaption".into()]),
            block: vec![
                Declaration::Property(Property {
                    property: "background".into(),
                    value: "hsl(0 0% 0% / 50%)".into(),
                }),
                Declaration::QualifiedRule(QualifiedRule {
                    selectors: Selectors(vec!["& > p".into()]),
                    block: vec![Declaration::Property(Property {
                        property: "font-size".into(),
                        value: ".9rem".into(),
                    })],
                }),
            ],
        }),
    ]);

    let transpiled_css = TranspiledCss::transpile(&["figure"], runtime_css);

    // figure { margin: 0; }
    // figure > figcaption { background: hsl(0 0% 0% / 50%); }
    // figure > figcaption > p { font-size: .9rem; }
    let expected_css = TranspiledCss(vec![
        Rule::QualifiedRule(QualifiedRule {
            selectors: Selectors(vec!["figure".into()]),
            block: vec![Declaration::Property(Property {
                property: "margin".into(),
                value: "0".into(),
            })],
        }),
        Rule::QualifiedRule(QualifiedRule {
            selectors: Selectors(vec!["figure > figcaption".into()]),
            block: vec![Declaration::Property(Property {
                property: "background".into(),
                value: "hsl(0 0% 0% / 50%)".into(),
            })],
        }),
        Rule::QualifiedRule(QualifiedRule {
            selectors: Selectors(vec!["figure > figcaption > p".into()]),
            block: vec![Declaration::Property(Property {
                property: "font-size".into(),
                value: ".9rem".into(),
            })],
        }),
    ]);

    assert_eq!(transpiled_css, expected_css);
}

#[test]
fn test_transpile_10() {
    // .foo {
    //   color: red;
    //   @nest & > .bar {
    //     color: blue;
    //   }
    // }
    let runtime_css = RuntimeCss(vec![
        Declaration::Property(Property {
            property: "color".into(),
            value: "red".into(),
        }),
        Declaration::AtRule(AtRule {
            rule_name: "nest".into(),
            rule_value: "& > .bar".into(),
            block: Some(vec![Declaration::Property(Property {
                property: "color".into(),
                value: "blue".into(),
            })]),
        }),
    ]);

    let transpiled_css = TranspiledCss::transpile(&[".foo"], runtime_css);

    // .foo { color: red; }
    // .foo > .bar { color: blue; }
    let expected_css = TranspiledCss(vec![
        Rule::QualifiedRule(QualifiedRule {
            selectors: Selectors(vec![".foo".into()]),
            block: vec![Declaration::Property(Property {
                property: "color".into(),
                value: "red".into(),
            })],
        }),
        Rule::QualifiedRule(QualifiedRule {
            selectors: Selectors(vec![".foo > .bar".into()]),
            block: vec![Declaration::Property(Property {
                property: "color".into(),
                value: "blue".into(),
            })],
        }),
    ]);

    assert_eq!(transpiled_css, expected_css);
}

#[test]
fn test_transpile_11() {
    // .foo {
    //   color: red;
    //   @nest .parent & {
    //     color: blue;
    //   }
    // }
    let runtime_css = RuntimeCss(vec![
        Declaration::Property(Property {
            property: "color".into(),
            value: "red".into(),
        }),
        Declaration::AtRule(AtRule {
            rule_name: "nest".into(),
            rule_value: ".parent &".into(),
            block: Some(vec![Declaration::Property(Property {
                property: "color".into(),
                value: "blue".into(),
            })]),
        }),
    ]);

    let transpiled_css = TranspiledCss::transpile(&[".foo"], runtime_css);

    // .foo { color: red; }
    // .parent .foo { color: blue; }
    let expected_css = TranspiledCss(vec![
        Rule::QualifiedRule(QualifiedRule {
            selectors: Selectors(vec![".foo".into()]),
            block: vec![Declaration::Property(Property {
                property: "color".into(),
                value: "red".into(),
            })],
        }),
        Rule::QualifiedRule(QualifiedRule {
            selectors: Selectors(vec![".parent .foo".into()]),
            block: vec![Declaration::Property(Property {
                property: "color".into(),
                value: "blue".into(),
            })],
        }),
    ]);

    assert_eq!(transpiled_css, expected_css);
}

#[test]
fn test_transpile_12() {
    // .foo {
    //   color: red;
    //   @nest :not(&) {
    //     color: blue;
    //   }
    // }
    let runtime_css = RuntimeCss(vec![
        Declaration::Property(Property {
            property: "color".into(),
            value: "red".into(),
        }),
        Declaration::AtRule(AtRule {
            rule_name: "nest".into(),
            rule_value: ":not(&)".into(),
            block: Some(vec![Declaration::Property(Property {
                property: "color".into(),
                value: "blue".into(),
            })]),
        }),
    ]);

    let transpiled_css = TranspiledCss::transpile(&[".foo"], runtime_css);

    // .foo { color: red; }
    // :not(.foo) { color: blue; }
    let expected_css = TranspiledCss(vec![
        Rule::QualifiedRule(QualifiedRule {
            selectors: Selectors(vec![".foo".into()]),
            block: vec![Declaration::Property(Property {
                property: "color".into(),
                value: "red".into(),
            })],
        }),
        Rule::QualifiedRule(QualifiedRule {
            selectors: Selectors(vec![":not(.foo)".into()]),
            block: vec![Declaration::Property(Property {
                property: "color".into(),
                value: "blue".into(),
            })],
        }),
    ]);

    assert_eq!(transpiled_css, expected_css);
}

#[test]
fn test_transpile_13() {
    // .foo {
    // color: blue;
    //   @nest .bar & {
    //     color: red;
    //     &.baz {
    //       color: green;
    //     }
    //   }
    // }
    let runtime_css = RuntimeCss(vec![
        Declaration::Property(Property {
            property: "color".into(),
            value: "blue".into(),
        }),
        Declaration::AtRule(AtRule {
            rule_name: "nest".into(),
            rule_value: ".bar &".into(),
            block: Some(vec![
                Declaration::Property(Property {
                    property: "color".into(),
                    value: "red".into(),
                }),
                Declaration::QualifiedRule(QualifiedRule {
                    selectors: Selectors(vec!["&.baz".into()]),
                    block: vec![Declaration::Property(Property {
                        property: "color".into(),
                        value: "green".into(),
                    })],
                }),
            ]),
        }),
    ]);

    let transpiled_css = TranspiledCss::transpile(&[".foo"], runtime_css);

    // .foo { color: blue; }
    // .bar .foo { color: red; }
    // .bar .foo.baz { color: green; }
    let expected_css = TranspiledCss(vec![
        Rule::QualifiedRule(QualifiedRule {
            selectors: Selectors(vec![".foo".into()]),
            block: vec![Declaration::Property(Property {
                property: "color".into(),
                value: "blue".into(),
            })],
        }),
        Rule::QualifiedRule(QualifiedRule {
            selectors: Selectors(vec![".bar .foo".into()]),
            block: vec![Declaration::Property(Property {
                property: "color".into(),
                value: "red".into(),
            })],
        }),
        Rule::QualifiedRule(QualifiedRule {
            selectors: Selectors(vec![".bar .foo.baz".into()]),
            block: vec![Declaration::Property(Property {
                property: "color".into(),
                value: "green".into(),
            })],
        }),
    ]);

    assert_eq!(transpiled_css, expected_css);
}

#[test]
fn test_transpile_14() {
    // .foo {
    //   display: grid;
    //   @media (orientation: landscape) {
    //     grid-auto-flow: column;
    //   }
    // }
    let runtime_css = RuntimeCss(vec![
        Declaration::Property(Property {
            property: "display".into(),
            value: "grid".into(),
        }),
        Declaration::AtRule(AtRule {
            rule_name: "media".into(),
            rule_value: "(orientation: landscape)".into(),
            block: Some(vec![Declaration::Property(Property {
                property: "grid-auto-flow".into(),
                value: "column".into(),
            })]),
        }),
    ]);

    let transpiled_css = TranspiledCss::transpile(&[".foo"], runtime_css);

    // .foo { display: grid; }
    // @media (orientation: landscape) {
    //   .foo {
    //     grid-auto-flow: column;
    //   }
    // }
    let expected_css = TranspiledCss(vec![
        Rule::QualifiedRule(QualifiedRule {
            selectors: Selectors(vec![".foo".into()]),
            block: vec![Declaration::Property(Property {
                property: "display".into(),
                value: "grid".into(),
            })],
        }),
        Rule::AtRule(AtRule {
            rule_name: "media".into(),
            rule_value: "(orientation: landscape)".into(),
            block: Some(vec![Declaration::QualifiedRule(QualifiedRule {
                selectors: Selectors(vec![".foo".into()]),
                block: vec![Declaration::Property(Property {
                    property: "grid-auto-flow".into(),
                    value: "column".into(),
                })],
            })]),
        }),
    ]);

    assert_eq!(transpiled_css, expected_css);
}

#[test]
fn test_transpile_15() {
    // .foo {
    //   display: grid;
    //   @media (orientation: landscape) {
    //     grid-auto-flow: column;
    //     @media (min-width > 1024px) {
    //       max-inline-size: 1024px;
    //     }
    //   }
    // }
    let runtime_css = RuntimeCss(vec![
        Declaration::Property(Property {
            property: "display".into(),
            value: "grid".into(),
        }),
        Declaration::AtRule(AtRule {
            rule_name: "media".into(),
            rule_value: "(orientation: landscape)".into(),
            block: Some(vec![
                Declaration::Property(Property {
                    property: "grid-auto-flow".into(),
                    value: "column".into(),
                }),
                Declaration::AtRule(AtRule {
                    rule_name: "media".into(),
                    rule_value: "(min-width > 1024px)".into(),
                    block: Some(vec![Declaration::Property(Property {
                        property: "max-inline-size".into(),
                        value: "1024px".into(),
                    })]),
                }),
            ]),
        }),
    ]);

    let transpiled_css = TranspiledCss::transpile(&[".foo"], runtime_css);

    // .foo { display: grid; }
    // @media (orientation: landscape) {
    //   .foo {
    //     grid-auto-flow: column;
    //   }
    // }
    // @media (orientation: landscape) and (min-width > 1024px) {
    //   .foo {
    //     max-inline-size: 1024px;
    //   }
    // }
    let expected_css = TranspiledCss(vec![
        Rule::QualifiedRule(QualifiedRule {
            selectors: Selectors(vec![".foo".into()]),
            block: vec![Declaration::Property(Property {
                property: "display".into(),
                value: "grid".into(),
            })],
        }),
        Rule::AtRule(AtRule {
            rule_name: "media".into(),
            rule_value: "(orientation: landscape)".into(),
            block: Some(vec![Declaration::QualifiedRule(QualifiedRule {
                selectors: Selectors(vec![".foo".into()]),
                block: vec![Declaration::Property(Property {
                    property: "grid-auto-flow".into(),
                    value: "column".into(),
                })],
            })]),
        }),
        Rule::AtRule(AtRule {
            rule_name: "media".into(),
            rule_value: "(orientation: landscape) and (min-width > 1024px)".into(),
            block: Some(vec![Declaration::QualifiedRule(QualifiedRule {
                selectors: Selectors(vec![".foo".into()]),
                block: vec![Declaration::Property(Property {
                    property: "max-inline-size".into(),
                    value: "1024px".into(),
                })],
            })]),
        }),
    ]);

    assert_eq!(transpiled_css, expected_css);
}

#[test]
fn test_to_style_string_1() {
    // .foo { color: blue; }
    // .foo > .bar { color: red; }
    let transpiled_css = TranspiledCss(vec![
        Rule::QualifiedRule(QualifiedRule {
            selectors: Selectors(vec![".foo".into()]),
            block: vec![Declaration::Property(Property {
                property: "color".into(),
                value: "blue".into(),
            })],
        }),
        Rule::QualifiedRule(QualifiedRule {
            selectors: Selectors(vec![".foo > .bar".into()]),
            block: vec![Declaration::Property(Property {
                property: "color".into(),
                value: "red".into(),
            })],
        }),
    ]);

    let transpiled_style = transpiled_css.to_style_string();

    let expected_style = ".foo{color:blue;}.foo > .bar{color:red;}";

    assert_eq!(transpiled_style, expected_style);
}

#[test]
fn test_to_style_string_2() {
    // .foo { color: blue; }
    // .foo.bar { color: red; }
    let transpiled_css = TranspiledCss(vec![
        Rule::QualifiedRule(QualifiedRule {
            selectors: Selectors(vec![".foo".into()]),
            block: vec![Declaration::Property(Property {
                property: "color".into(),
                value: "blue".into(),
            })],
        }),
        Rule::QualifiedRule(QualifiedRule {
            selectors: Selectors(vec![".foo.bar".into()]),
            block: vec![Declaration::Property(Property {
                property: "color".into(),
                value: "red".into(),
            })],
        }),
    ]);

    let transpiled_style = transpiled_css.to_style_string();

    let expected_style = ".foo{color:blue;}.foo.bar{color:red;}";

    assert_eq!(transpiled_style, expected_style);
}

#[test]
fn test_to_style_string_3() {
    // .foo, .bar { color: blue; }
    // :is(.foo, .bar) + .baz,
    // :is(.foo, .bar).qux { color: red; }
    let transpiled_css = TranspiledCss(vec![
        Rule::QualifiedRule(QualifiedRule {
            selectors: Selectors(vec![".foo".into(), ".bar".into()]),
            block: vec![Declaration::Property(Property {
                property: "color".into(),
                value: "blue".into(),
            })],
        }),
        Rule::QualifiedRule(QualifiedRule {
            selectors: Selectors(vec![
                ":is(.foo, .bar) + .baz".into(),
                ":is(.foo, .bar).qux".into(),
            ]),
            block: vec![Declaration::Property(Property {
                property: "color".into(),
                value: "red".into(),
            })],
        }),
    ]);

    let transpiled_style = transpiled_css.to_style_string();

    let expected_style =
        ".foo,.bar{color:blue;}:is(.foo, .bar) + .baz,:is(.foo, .bar).qux{color:red;}";

    assert_eq!(transpiled_style, expected_style);
}

#[test]
fn test_to_style_string_4() {
    // .foo { color: blue; }
    // .foo .bar .foo .baz .foo .qux { color: red; }
    let transpiled_css = TranspiledCss(vec![
        Rule::QualifiedRule(QualifiedRule {
            selectors: Selectors(vec![".foo".into()]),
            block: vec![Declaration::Property(Property {
                property: "color".into(),
                value: "blue".into(),
            })],
        }),
        Rule::QualifiedRule(QualifiedRule {
            selectors: Selectors(vec![".foo .bar .foo .baz .foo .qux".into()]),
            block: vec![Declaration::Property(Property {
                property: "color".into(),
                value: "red".into(),
            })],
        }),
    ]);

    let transpiled_style = transpiled_css.to_style_string();

    let expected_style = ".foo{color:blue;}.foo .bar .foo .baz .foo .qux{color:red;}";

    assert_eq!(transpiled_style, expected_style);
}

#[test]
fn test_to_style_string_5() {
    // .foo {
    //   color: blue;
    //   padding: 2ch;
    // }
    let transpiled_css = TranspiledCss(vec![Rule::QualifiedRule(QualifiedRule {
        selectors: Selectors(vec![".foo".into()]),
        block: vec![
            Declaration::Property(Property {
                property: "color".into(),
                value: "blue".into(),
            }),
            Declaration::Property(Property {
                property: "padding".into(),
                value: "2ch".into(),
            }),
        ],
    })]);

    let transpiled_style = transpiled_css.to_style_string();

    let expected_style = ".foo{color:blue;padding:2ch;}";

    assert_eq!(transpiled_style, expected_style);
}

#[test]
fn test_to_style_string_6() {
    // .foo { color: blue; }
    // .foo.foo { padding: 2ch; }
    let transpiled_css = TranspiledCss(vec![
        Rule::QualifiedRule(QualifiedRule {
            selectors: Selectors(vec![".foo".into()]),
            block: vec![Declaration::Property(Property {
                property: "color".into(),
                value: "blue".into(),
            })],
        }),
        Rule::QualifiedRule(QualifiedRule {
            selectors: Selectors(vec![".foo.foo".into()]),
            block: vec![Declaration::Property(Property {
                property: "padding".into(),
                value: "2ch".into(),
            })],
        }),
    ]);

    let transpiled_style = transpiled_css.to_style_string();

    let expected_style = ".foo{color:blue;}.foo.foo{padding:2ch;}";

    assert_eq!(transpiled_style, expected_style);
}

#[test]
fn test_to_style_string_7() {
    // :is(.error, #404):hover > .baz { color: red; }
    let transpiled_css = TranspiledCss(vec![Rule::QualifiedRule(QualifiedRule {
        selectors: Selectors(vec![":is(.error, #404):hover > .baz".into()]),
        block: vec![Declaration::Property(Property {
            property: "color".into(),
            value: "red".into(),
        })],
    })]);

    let transpiled_style = transpiled_css.to_style_string();

    let expected_style = ":is(.error, #404):hover > .baz{color:red;}";

    assert_eq!(transpiled_style, expected_style);
}

#[test]
fn test_to_style_string_8() {
    // .foo:is(.bar, .foo.baz) { color: red; }
    let transpiled_css = TranspiledCss(vec![Rule::QualifiedRule(QualifiedRule {
        selectors: Selectors(vec![".foo:is(.bar, .foo.baz)".into()]),
        block: vec![Declaration::Property(Property {
            property: "color".into(),
            value: "red".into(),
        })],
    })]);

    let transpiled_style = transpiled_css.to_style_string();

    let expected_style = ".foo:is(.bar, .foo.baz){color:red;}";

    assert_eq!(transpiled_style, expected_style);
}

#[test]
fn test_to_style_string_9() {
    // figure { margin: 0; }
    // figure > figcaption { background: hsl(0 0% 0% / 50%); }
    // figure > figcaption > p { font-size: .9rem; }
    let transpiled_css = TranspiledCss(vec![
        Rule::QualifiedRule(QualifiedRule {
            selectors: Selectors(vec!["figure".into()]),
            block: vec![Declaration::Property(Property {
                property: "margin".into(),
                value: "0".into(),
            })],
        }),
        Rule::QualifiedRule(QualifiedRule {
            selectors: Selectors(vec!["figure > figcaption".into()]),
            block: vec![Declaration::Property(Property {
                property: "background".into(),
                value: "hsl(0 0% 0% / 50%)".into(),
            })],
        }),
        Rule::QualifiedRule(QualifiedRule {
            selectors: Selectors(vec!["figure > figcaption > p".into()]),
            block: vec![Declaration::Property(Property {
                property: "font-size".into(),
                value: ".9rem".into(),
            })],
        }),
    ]);

    let transpiled_style = transpiled_css.to_style_string();

    let expected_style = "figure{margin:0;}figure > figcaption{background:hsl(0 0% 0% / 50%);}figure > figcaption > p{font-size:.9rem;}";

    assert_eq!(transpiled_style, expected_style);
}

#[test]
fn test_to_style_string_10() {
    // .foo { color: red; }
    // .foo > .bar { color: blue; }
    let transpiled_css = TranspiledCss(vec![
        Rule::QualifiedRule(QualifiedRule {
            selectors: Selectors(vec![".foo".into()]),
            block: vec![Declaration::Property(Property {
                property: "color".into(),
                value: "red".into(),
            })],
        }),
        Rule::QualifiedRule(QualifiedRule {
            selectors: Selectors(vec![".foo > .bar".into()]),
            block: vec![Declaration::Property(Property {
                property: "color".into(),
                value: "blue".into(),
            })],
        }),
    ]);

    let transpiled_style = transpiled_css.to_style_string();

    let expected_style = ".foo{color:red;}.foo > .bar{color:blue;}";

    assert_eq!(transpiled_style, expected_style);
}

#[test]
fn test_to_style_string_11() {
    // .foo { color: red; }
    // .parent .foo { color: blue; }
    let transpiled_css = TranspiledCss(vec![
        Rule::QualifiedRule(QualifiedRule {
            selectors: Selectors(vec![".foo".into()]),
            block: vec![Declaration::Property(Property {
                property: "color".into(),
                value: "red".into(),
            })],
        }),
        Rule::QualifiedRule(QualifiedRule {
            selectors: Selectors(vec![".parent .foo".into()]),
            block: vec![Declaration::Property(Property {
                property: "color".into(),
                value: "blue".into(),
            })],
        }),
    ]);

    let transpiled_style = transpiled_css.to_style_string();

    let expected_style = ".foo{color:red;}.parent .foo{color:blue;}";

    assert_eq!(transpiled_style, expected_style);
}

#[test]
fn test_to_style_string_12() {
    // .foo { color: red; }
    // :not(.foo) { color: blue; }
    let transpiled_css = TranspiledCss(vec![
        Rule::QualifiedRule(QualifiedRule {
            selectors: Selectors(vec![".foo".into()]),
            block: vec![Declaration::Property(Property {
                property: "color".into(),
                value: "red".into(),
            })],
        }),
        Rule::QualifiedRule(QualifiedRule {
            selectors: Selectors(vec![":not(.foo)".into()]),
            block: vec![Declaration::Property(Property {
                property: "color".into(),
                value: "blue".into(),
            })],
        }),
    ]);

    let transpiled_style = transpiled_css.to_style_string();

    let expected_style = ".foo{color:red;}:not(.foo){color:blue;}";

    assert_eq!(transpiled_style, expected_style);
}

#[test]
fn test_to_style_string_13() {
    // .foo { color: blue; }
    // .bar .foo { color: red; }
    // .bar .foo.baz { color: green; }
    let transpiled_css = TranspiledCss(vec![
        Rule::QualifiedRule(QualifiedRule {
            selectors: Selectors(vec![".foo".into()]),
            block: vec![Declaration::Property(Property {
                property: "color".into(),
                value: "blue".into(),
            })],
        }),
        Rule::QualifiedRule(QualifiedRule {
            selectors: Selectors(vec![".bar .foo".into()]),
            block: vec![Declaration::Property(Property {
                property: "color".into(),
                value: "red".into(),
            })],
        }),
        Rule::QualifiedRule(QualifiedRule {
            selectors: Selectors(vec![".bar .foo.baz".into()]),
            block: vec![Declaration::Property(Property {
                property: "color".into(),
                value: "green".into(),
            })],
        }),
    ]);

    let transpiled_style = transpiled_css.to_style_string();

    let expected_style = ".foo{color:blue;}.bar .foo{color:red;}.bar .foo.baz{color:green;}";

    assert_eq!(transpiled_style, expected_style);
}

#[test]
fn test_to_style_string_14() {
    // .foo { display: grid; }
    // @media (orientation: landscape) {
    //   .foo {
    //     grid-auto-flow: column;
    //   }
    // }
    let transpiled_css = TranspiledCss(vec![
        Rule::QualifiedRule(QualifiedRule {
            selectors: Selectors(vec![".foo".into()]),
            block: vec![Declaration::Property(Property {
                property: "display".into(),
                value: "grid".into(),
            })],
        }),
        Rule::AtRule(AtRule {
            rule_name: "media".into(),
            rule_value: "(orientation: landscape)".into(),
            block: Some(vec![Declaration::QualifiedRule(QualifiedRule {
                selectors: Selectors(vec![".foo".into()]),
                block: vec![Declaration::Property(Property {
                    property: "grid-auto-flow".into(),
                    value: "column".into(),
                })],
            })]),
        }),
    ]);

    let transpiled_style = transpiled_css.to_style_string();

    let expected_style =
        ".foo{display:grid;}@media (orientation: landscape){.foo{grid-auto-flow:column;}}";

    assert_eq!(transpiled_style, expected_style);
}

#[test]
fn test_to_style_string_15() {
    // .foo { display: grid; }
    // @media (orientation: landscape) {
    //   .foo {
    //     grid-auto-flow: column;
    //   }
    // }
    // @media (orientation: landscape) and (min-width > 1024px) {
    //   .foo {
    //     max-inline-size: 1024px;
    //   }
    // }
    let transpiled_css = TranspiledCss(vec![
        Rule::QualifiedRule(QualifiedRule {
            selectors: Selectors(vec![".foo".into()]),
            block: vec![Declaration::Property(Property {
                property: "display".into(),
                value: "grid".into(),
            })],
        }),
        Rule::AtRule(AtRule {
            rule_name: "media".into(),
            rule_value: "(orientation: landscape)".into(),
            block: Some(vec![Declaration::QualifiedRule(QualifiedRule {
                selectors: Selectors(vec![".foo".into()]),
                block: vec![Declaration::Property(Property {
                    property: "grid-auto-flow".into(),
                    value: "column".into(),
                })],
            })]),
        }),
        Rule::AtRule(AtRule {
            rule_name: "media".into(),
            rule_value: "(orientation: landscape) and (min-width > 1024px)".into(),
            block: Some(vec![Declaration::QualifiedRule(QualifiedRule {
                selectors: Selectors(vec![".foo".into()]),
                block: vec![Declaration::Property(Property {
                    property: "max-inline-size".into(),
                    value: "1024px".into(),
                })],
            })]),
        }),
    ]);

    let transpiled_style = transpiled_css.to_style_string();

    let expected_style =
            ".foo{display:grid;}@media (orientation: landscape){.foo{grid-auto-flow:column;}}@media (orientation: landscape) and (min-width > 1024px){.foo{max-inline-size:1024px;}}";

    assert_eq!(transpiled_style, expected_style);
}

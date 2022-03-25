use crate::ast::*;
use crate::cursor::*;

// recursive depth first property normalize
//
// eg)
// .bar {
//     @media (orientation: landscape) {
//         & .foo {
//             background: black;
//             color: red;
//         }
//     }
// }
//
// to
//
// media (orientation: landscape) {
//     .bar .foo {
//         background: black;
//     }
// }
// media (orientation: landscape) {
//     .bar .foo {
//         color: red;
//     }
// }
fn transpile_declarations(
    selectors: Selectors,
    at_rules: Vec<(String, String)>,
    declarations: Vec<Declaration>,
) -> Vec<Declaration> {
    let mut return_declarations = vec![];

    // flag to ignore after nest property
    let mut nest_flag = false;

    for declaration in declarations {
        match declaration {
            Declaration::Property(property) => {
                if nest_flag {
                    continue;
                }

                // Property with selectors
                let mut at_rules = at_rules.clone();
                let mut rule = Declaration::QualifiedRule(QualifiedRule {
                    selectors: selectors.clone(),
                    block: vec![Declaration::Property(property)],
                });

                // Nesting at-rules stack
                while let Some(at_rule) = at_rules.pop() {
                    rule = Declaration::AtRule(AtRule {
                        rule_name: at_rule.0,
                        rule_value: at_rule.1,
                        block: Some(vec![rule]),
                    });
                }

                // push rule declaration
                return_declarations.push(rule);
            }
            Declaration::AtRule(at_rule) => {
                nest_flag = true;

                let mut at_rules = at_rules.clone();
                if &at_rule.rule_name == "nest" {
                    // @nest rule

                    // perse rule value as selectors
                    let content = at_rule.rule_value + "{}";
                    let mut cursor = Cursor::new(&content);
                    if let Some(at_rule_selectors) = cursor.parse_selectors() {
                        // parent_selector is left as is if the parent has a single selector,
                        // or wrapped in :is() if there are multiple comma-separated selectors.
                        let parent_selector = if selectors.0.len() == 1 {
                            selectors.0[0].clone()
                        } else {
                            String::from(":is(") + &selectors.0.join(", ") + ")"
                        };

                        // replace `&` char with parent_selector
                        let mut selectors = vec![];
                        for s in at_rule_selectors.0.iter() {
                            let mut selector = String::new();
                            for ch in s.chars() {
                                if ch == '&' {
                                    selector += &parent_selector;
                                } else {
                                    selector.push(ch);
                                }
                            }
                            selectors.push(selector);
                        }
                        let selectors = Selectors(selectors);

                        // transpile inner block and append
                        return_declarations.append(&mut transpile_declarations(
                            selectors,
                            at_rules,
                            at_rule.block.unwrap_or_default(),
                        ));
                    } else {
                        // ignore error
                    }
                } else {
                    // other at-rule
                    if let Some(last_at_rule) = at_rules.pop() {
                        // if same at-rule name with parent at-rule, then merge at-rules.
                        // else push at-rule into at-rules stack
                        if last_at_rule.0 == at_rule.rule_name {
                            // remove outer paren of at-rule value
                            let mut cursor = Cursor::new(&last_at_rule.1);
                            let last_at_rule = if let Some(value) = cursor.take_paren() {
                                if cursor.is_empty() {
                                    value
                                        .trim_start_matches('(')
                                        .trim_end_matches(')')
                                        .to_string()
                                } else {
                                    last_at_rule.1
                                }
                            } else {
                                last_at_rule.1
                            };

                            // remove outer paren of at-rule value
                            let mut cursor = Cursor::new(&at_rule.rule_value);
                            let rule_value = if let Some(value) = cursor.take_paren() {
                                if cursor.is_empty() {
                                    value
                                        .trim_start_matches('(')
                                        .trim_end_matches(')')
                                        .to_string()
                                } else {
                                    at_rule.rule_value
                                }
                            } else {
                                at_rule.rule_value
                            };

                            // merge at rule
                            let merged_at_rule = (
                                at_rule.rule_name,
                                String::from("(") + &last_at_rule + ") and (" + &rule_value + ")",
                            );
                            at_rules.push(merged_at_rule);
                        } else {
                            at_rules.push(last_at_rule);
                            at_rules.push((at_rule.rule_name, at_rule.rule_value));
                        }
                    } else {
                        at_rules.push((at_rule.rule_name, at_rule.rule_value));
                    }

                    // transpile inner block and append
                    return_declarations.append(&mut transpile_declarations(
                        selectors.clone(),
                        at_rules,
                        at_rule.block.unwrap_or_default(),
                    ));
                }
            }
            Declaration::QualifiedRule(rule) => {
                nest_flag = true;

                // parent_selector is left as is if the parent has a single selector,
                // or wrapped in :is() if there are multiple comma-separated selectors.
                let parent_selector = if selectors.0.len() == 1 {
                    selectors.0[0].clone()
                } else {
                    String::from(":is(") + &selectors.0.join(", ") + ")"
                };

                // replace `&` char with parent_selector
                let mut selectors = vec![];
                for s in rule.selectors.0.iter() {
                    if !s.starts_with('&') {
                        continue;
                    }
                    let mut selector = String::new();
                    for ch in s.chars() {
                        if ch == '&' {
                            selector += &parent_selector;
                        } else {
                            selector.push(ch);
                        }
                    }
                    selectors.push(selector);
                }
                let selectors = Selectors(selectors);

                // transpile inner block and append
                return_declarations.append(&mut transpile_declarations(
                    selectors,
                    at_rules.clone(),
                    rule.block,
                ));
            }
        }
    }
    return_declarations
}

// Root CSS Component
#[cfg_attr(test, derive(Debug, PartialEq, Eq))]
pub enum Rule {
    QualifiedRule(QualifiedRule),
    AtRule(AtRule),
}
fn into_rules(declarations: Vec<Declaration>) -> Vec<Rule> {
    declarations
        .into_iter()
        .map(|d| match d {
            Declaration::AtRule(at_rule) => Rule::AtRule(at_rule),
            Declaration::QualifiedRule(rule) => Rule::QualifiedRule(rule),
            Declaration::Property(_) => panic!("Error for top level property!"),
        })
        .collect()
}

// transpile CSS Nesting
// first normalize property and then merge same selector properties
// ignore errors
fn merge_same_selectors(rules: Vec<Rule>) -> TranspiledCss {
    let mut merged_rules = vec![];
    let mut at_rules_map = vec![];
    let mut qualified_rules_map = vec![];
    for rule in rules {
        match rule {
            Rule::QualifiedRule(rule) => {
                if let Some((_, v)) = qualified_rules_map
                    .iter_mut()
                    .find(|t: &&mut (Selectors, Vec<QualifiedRule>)| t.0 == rule.selectors)
                {
                    v.push(rule);
                } else {
                    qualified_rules_map.push((rule.selectors.clone(), vec![rule]));
                }
            }
            Rule::AtRule(at_rule) => {
                if let Some((_, v)) =
                    at_rules_map
                        .iter_mut()
                        .find(|t: &&mut ((String, String), Vec<AtRule>)| {
                            t.0 == (at_rule.rule_name.clone(), at_rule.rule_value.clone())
                        })
                {
                    v.push(at_rule);
                } else {
                    at_rules_map.push((
                        (at_rule.rule_name.clone(), at_rule.rule_value.clone()),
                        vec![at_rule],
                    ));
                }
            }
        }
    }
    for (selectors, v) in qualified_rules_map {
        let mut block = vec![];
        for mut qualified_rule in v {
            block.append(&mut qualified_rule.block);
        }
        merged_rules.push(Rule::QualifiedRule(QualifiedRule { selectors, block }));
    }
    for ((rule_name, rule_value), v) in at_rules_map {
        let mut block = vec![];
        for at_rule in v {
            if let Some(mut b) = at_rule.block {
                block.append(&mut b);
            } else {
                merged_rules.push(Rule::AtRule(at_rule));
            }
        }
        if !block.is_empty() {
            merged_rules.push(Rule::AtRule(AtRule {
                rule_name,
                rule_value,
                block: Some(block),
            }));
        }
    }

    TranspiledCss(merged_rules)
}

fn declaration_to_style_string(declaration: Declaration) -> String {
    match declaration {
        Declaration::AtRule(at_rule) => {
            if let Some(block) = at_rule.block {
                String::new()
                    + "@"
                    + &at_rule.rule_name
                    + " "
                    + &at_rule.rule_value
                    + "{"
                    + &block
                        .into_iter()
                        .map(|d| declaration_to_style_string(d))
                        .collect::<Vec<_>>()
                        .join("")
                    + "}"
            } else {
                String::new() + "@" + &at_rule.rule_name + " " + &at_rule.rule_value + ";"
            }
        }
        Declaration::QualifiedRule(rule) => {
            String::new()
                + &rule.selectors.0.join(",")
                + "{"
                + &rule
                    .block
                    .into_iter()
                    .map(|d| declaration_to_style_string(d))
                    .collect::<Vec<_>>()
                    .join("")
                + "}"
        }
        Declaration::Property(property) => {
            String::new() + &property.property + ":" + &property.value + ";"
        }
    }
} // --- Transpiled Css ---

#[cfg_attr(test, derive(Debug, PartialEq, Eq))]
pub struct TranspiledCss(pub Vec<Rule>);
impl TranspiledCss {
    pub fn transpile(classes: &[impl ToString], runtime_css: RuntimeCss) -> TranspiledCss {
        let selectors = classes.iter().map(|c| c.to_string()).collect::<Vec<_>>();
        let selectors = Selectors(selectors);
        let declarations = transpile_declarations(selectors, vec![], runtime_css.0);
        let rules = into_rules(declarations);
        merge_same_selectors(rules)
    }

    pub fn to_style_string(self) -> String {
        let mut style_string = String::new();

        for rule in self.0 {
            match rule {
                Rule::AtRule(at_rule) => {
                    style_string
                        .push_str(&declaration_to_style_string(Declaration::AtRule(at_rule)));
                }
                Rule::QualifiedRule(rule) => {
                    style_string.push_str(&declaration_to_style_string(
                        Declaration::QualifiedRule(rule),
                    ));
                }
            }
        }

        style_string
    }
}

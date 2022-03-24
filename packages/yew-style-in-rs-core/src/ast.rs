use crate::cursor::*;

// property: value;
//
// eg)
// background: black;
#[cfg_attr(test, derive(Debug, PartialEq, Eq))]
pub struct Property {
    pub property: String,
    pub value: String,
}

// One of property, at-rule, qualified-rule, runtime-parsed-declaration
#[cfg_attr(test, derive(Debug, PartialEq, Eq))]
pub enum Declaration {
    Property(Property),
    AtRule(AtRule),
    QualifiedRule(QualifiedRule),
}

// selector1 selector2 {
//     ...
// }
//
// eg)
// .foo .bar {
//     ...
// }
#[cfg_attr(test, derive(Debug))]
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Selectors(pub Vec<String>);

// selectors {
//     block
// }
//
// eg)
// .foo .bar {
//     background: black;
// }
#[cfg_attr(test, derive(Debug, PartialEq, Eq))]
pub struct QualifiedRule {
    pub selectors: Selectors,
    pub block: Vec<Declaration>,
}

// @rule_name rule_value {
//     block
// }
//
// eg)
// @media (orientation: landscape) and (min-width: 480px) {
//     background: black;
// }
#[cfg_attr(test, derive(Debug, PartialEq, Eq))]
pub struct AtRule {
    pub rule_name: String,
    pub rule_value: String,
    pub block: Option<Vec<Declaration>>,
}

// contains only one qualified rule for runtime css
//
// .AbCdEfGh {
//     & .foo {}
// }
#[cfg_attr(test, derive(Debug, PartialEq, Eq))]
pub struct RuntimeCss(pub QualifiedRule);
impl RuntimeCss {
    // parse CSS code
    //
    // AbCdEfGh
    pub fn parse(class: impl ToString, code: impl ToString) -> Result<Self, Self> {
        let code = code.to_string();
        let mut cursor = Cursor::new(&code);

        match cursor.parse_declaration_list() {
            Ok(declarations) => Ok(RuntimeCss(QualifiedRule {
                selectors: Selectors(vec![class.to_string()]),
                block: declarations,
            })),
            Err(ParseError::Fatal) => Err(RuntimeCss(QualifiedRule {
                selectors: Selectors(vec![class.to_string()]),
                block: vec![],
            })),
            Err(ParseError::Ignorable(declarations)) => Err(RuntimeCss(QualifiedRule {
                selectors: Selectors(vec![class.to_string()]),
                block: declarations,
            })),
        }
    }
}

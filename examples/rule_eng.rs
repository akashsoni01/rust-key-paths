use std::{borrow::Cow, fmt::Debug};

use key_paths_derive::Kp;
use rust_key_paths::{AccessorTrait, KpType};

#[derive(Debug, PartialEq, Eq)]
pub enum RuleBuilderError<E: Debug + Clone + 'static + PartialEq + Eq> {
    ExampleError(Cow<'static, String>),
    Fail(E),
    Success
}
pub struct RuleBuilder<'a, R, V, E: Debug + Clone + 'static + PartialEq + Eq> {
    root: Option<&'a R>,
    kp: KpType<'a, R, V>,
    mandatory_rules: Vec<fn(Option<&'a V>) -> RuleBuilderError<E>>,
    rules: Vec<fn(Option<&'a V>) -> RuleBuilderError<E>>,
}

impl<'a, R, V, E> RuleBuilder<'a, R, V, E> 
where 
E: Debug + Clone + 'static + PartialEq + Eq
{
    pub fn new(kp: KpType<'a, R, V>) -> Self {
        Self {
            root: None,
            kp,
            rules: vec![],
            mandatory_rules: vec![]
        }
    }

    pub fn with_root(mut self, root: &'a R) -> Self {
        self.root = Some(root);
        self
    }

    pub fn rule(mut self, f: fn(Option<&'a V>) -> RuleBuilderError<E>) -> Self {
        self.rules.push(f);
        self
    }

    pub fn madatory_rule(mut self, f: fn(Option<&'a V>) -> RuleBuilderError<E>) -> Self {
        self.mandatory_rules.push(f);
        self
    }


    pub fn apply(&self) -> Vec<RuleBuilderError<E>> {
        let val = self.kp.get_optional(self.root);
        for rule in self.mandatory_rules.iter() {
            let result = rule(val);
            if  RuleBuilderError::Success != result {
                return vec![result];
            }
        }
        self.rules.iter().map(|f| f(val)).collect()
    }
}


mod iso_pain {
    type IsoError = crate::RuleBuilderError<String>;
    // For raw rule — still receives Option<&String>
    pub fn iso123rule<'a>(r: Option<&'a String>) -> IsoError {
        if r.map_or(true, |s| s.trim().is_empty()) {
            IsoError::Fail("123 rule failed".to_string())
        } else {
            IsoError::Success
        }
    }

    // For mandatory/optional — receives &String directly, None already handled
    pub fn not_blank<'a>(s: &'a String) -> IsoError {
        if s.trim().is_empty() {
            IsoError::Fail("blank field".to_string())
        } else {
            IsoError::Success
        }
    }

    pub fn max_len_35<'a>(s: &'a String) -> IsoError {
        if s.len() > 35 {
            IsoError::Fail("max_len_35".to_string())
        } else {
            IsoError::Success
        }
    }
}

#[derive(Kp)]
struct Test {
    a: String,
    b: String,

}

fn main() {
    let t = Test {
        a: "  ".to_string(),
        b: "asdf ".to_string(),
    };

    let rules = [RuleBuilder::new(Test::a())
        .with_root(&t)
        .rule(iso_pain::iso123rule)
        .rule(iso_pain::iso123rule)
        .rule(iso_pain::iso123rule)
        .rule(iso_pain::iso123rule)
        .madatory_rule(iso_pain::iso123rule),


        RuleBuilder::new(Test::b())
        .with_root(&t)
        .rule(iso_pain::iso123rule)
        .rule(iso_pain::iso123rule)
        .rule(iso_pain::iso123rule)
        .rule(iso_pain::iso123rule)
        .madatory_rule(iso_pain::iso123rule),
        ];
        // let errors = rules
        // .iter()
        // .fold(Vec::new(), |mut acc, v|  {acc.append(&mut v.apply()); acc} );

        let errors: Vec<RuleBuilderError<String>> = rules.iter().flat_map(|v| v.apply()).collect();
        
        for e in errors {
            println!("{:?}", e);
        }
}

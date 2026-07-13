use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MToken {
    pub text: String,
    pub tag: String,
    pub whitespace: String,
    pub phonemes: Option<String>,
    pub start_ts: Option<f64>,
    pub end_ts: Option<f64>,
    #[serde(rename = "_")]
    pub underscore: Option<Underscore>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Underscore {
    pub is_head: bool,
    pub alias: Option<String>,
    pub stress: Option<f64>,
    pub currency: Option<String>,
    pub num_flags: String,
    pub prespace: bool,
    pub rating: Option<i32>,
}

impl MToken {
    pub fn new(text: String, tag: String, whitespace: String) -> Self {
        Self {
            text,
            tag,
            whitespace,
            phonemes: None,
            start_ts: None,
            end_ts: None,
            underscore: Some(Underscore {
                is_head: true,
                num_flags: String::new(),
                prespace: false,
                ..Default::default()
            }),
        }
    }

    pub fn underscore_mut(&mut self) -> &mut Underscore {
        if self.underscore.is_none() {
            self.underscore = Some(Underscore::default());
        }
        self.underscore.as_mut().unwrap()
    }

    pub fn underscore(&self) -> &Underscore {
        static DEFAULT_UNDERSCORE: Underscore = Underscore {
            is_head: false,
            alias: None,
            stress: None,
            currency: None,
            num_flags: String::new(),
            prespace: false,
            rating: None,
        };
        self.underscore.as_ref().unwrap_or(&DEFAULT_UNDERSCORE)
    }
}

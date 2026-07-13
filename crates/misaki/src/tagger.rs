use serde::{Deserialize, Serialize};
use serde_json;
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct AveragedPerceptron {
    pub feature_weights: HashMap<String, HashMap<String, f32>>,
    pub classes: Vec<String>,
}

impl AveragedPerceptron {
    pub fn new(weights_json: &str, classes_txt: &str) -> Self {
        let feature_weights: HashMap<String, HashMap<String, f32>> = serde_json::from_str(weights_json)
            .unwrap_or_default();
        let classes: Vec<String> = classes_txt.lines().map(|s| s.trim().to_string()).collect();
        Self { feature_weights, classes }
    }

    pub fn predict(&self, word_features: HashMap<String, usize>) -> (&str, f32) {
        let mut scores: HashMap<&str, f32> = HashMap::new();
        for (feature, value) in word_features {
            if let Some(weights) = self.feature_weights.get(&feature)
                && value != 0 {
                    for (label, weight) in weights {
                        *scores.entry(label.as_str()).or_insert(0.0) += weight * (value as f32);
                    }
                }
        }

        let class = self.classes.iter()
            .max_by(|a, b| {
                let sa = scores.get(a.as_str()).unwrap_or(&0.0);
                let sb = scores.get(b.as_str()).unwrap_or(&0.0);
                sa.partial_cmp(sb).unwrap()
            })
            .unwrap();

        let max_score = *scores.get(class.as_str()).unwrap_or(&0.0);
        (class.as_str(), max_score)
    }
}

pub struct Tag<'a> {
    pub word: &'a str,
    pub tag: String,
    pub conf: f32,
}

pub struct PerceptronTagger {
    model: AveragedPerceptron,
    tags: HashMap<String, String>,
}

impl PerceptronTagger {
    pub fn new(weights_json: &str, classes_txt: &str, tags_json: &str) -> Self {
        let tags: HashMap<String, String> = serde_json::from_str(tags_json).unwrap_or_default();
        Self {
            model: AveragedPerceptron::new(weights_json, classes_txt),
            tags,
        }
    }

    pub fn tag<'a>(&'a self, words: &[&'a str]) -> Vec<Tag<'a>> {
        let mut prev = "-START-";
        let mut prev2 = "-START2-";
        let mut output = Vec::new();

        let mut context = Vec::new();
        context.push("-START-");
        context.push("-START2-");
        for &token in words {
            context.push(if token.contains("'-'") && !token.starts_with('-') {
                "!HYPHEN"
            } else if token.parse::<usize>().is_ok() && token.len() == 4 {
                "!YEAR"
            } else if token.chars().next().is_some_and(|c| c.is_ascii_digit()) {
                "!DIGITS"
            } else {
                token
            });
        }
        context.push("-END-");
        context.push("-END2-");

        for (i, &token) in words.iter().enumerate() {
            if let Some(tag) = self.tags.get(token) {
                output.push(Tag {
                    word: token,
                    tag: tag.clone(),
                    conf: 1.0,
                });
                prev2 = prev;
                prev = tag;
            } else {
                let features = Self::get_features(i + 2, token, &context, prev, prev2);
                let (tag, conf) = self.model.predict(features);
                output.push(Tag {
                    word: token,
                    tag: tag.to_string(),
                    conf,
                });
                prev2 = prev;
                prev = tag;
            }
        }
        output
    }

    fn get_features(i: usize, word: &str, context: &[&str], prev: &str, prev2: &str) -> HashMap<String, usize> {
        let mut features = HashMap::new();
        features.insert("bias".to_string(), 1);

        let suffix: String = word.chars().rev().take(3).collect::<String>().chars().rev().collect();
        features.insert(format!("i suffix {}", suffix), 1);

        let pref1: String = word.chars().take(1).collect();
        features.insert(format!("i pref1 {}", pref1), 1);

        features.insert(format!("i-1 tag {}", prev), 1);
        features.insert(format!("i-2 tag {}", prev2), 1);
        features.insert(format!("i tag+i-2 tag {} {}", prev, prev2), 1);
        features.insert(format!("i word {}", context[i]), 1);
        features.insert(format!("i-1 tag+i word {} {}", prev, context[i]), 1);
        features.insert(format!("i-1 word {}", context[i - 1]), 1);
        features.insert(format!("i-2 word {}", context[i - 2]), 1);
        features.insert(format!("i+1 word {}", context[i + 1]), 1);
        features.insert(format!("i+2 word {}", context[i + 2]), 1);

        let i_plus_1_suffix: String = context[i + 1].chars().rev().take(3).collect::<String>().chars().rev().collect();
        features.insert(format!("i+1 suffix {}", i_plus_1_suffix), 1);

        let i_minus_1_suffix: String = context[i - 1].chars().rev().take(3).collect::<String>().chars().rev().collect();
        features.insert(format!("i-1 suffix {}", i_minus_1_suffix), 1);

        features
    }
}

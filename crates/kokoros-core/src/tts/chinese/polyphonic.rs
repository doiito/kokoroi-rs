use std::collections::HashMap;
use lazy_static::lazy_static;

lazy_static! {
    static ref POLYPHONIC_DICT: HashMap<char, Vec<(&'static str, &'static str)>> = {
        let mut m = HashMap::new();
        
        m.insert('行', vec![
            ("银行", "hang2"),
            ("行走", "xing2"),
            ("行人", "xing2"),
            ("行为", "xing2"),
            ("行动", "xing2"),
            ("行列", "hang2"),
            ("行业", "hang2"),
            ("行情", "hang2"),
            ("行李", "xing2"),
        ]);
        
        m.insert('重', vec![
            ("重要", "zhong4"),
            ("重量", "zhong4"),
            ("重复", "chong2"),
            ("重新", "chong2"),
            ("重合", "chong2"),
            ("沉重", "zhong4"),
            ("严重", "zhong4"),
        ]);
        
        m.insert('长', vec![
            ("长度", "chang2"),
            ("长短", "chang2"),
            ("长辈", "zhang3"),
            ("成长", "zhang3"),
            ("生长", "zhang3"),
            ("家长", "zhang3"),
            ("市长", "zhang3"),
            ("长期", "chang2"),
        ]);
        
        m.insert('数', vec![
            ("数学", "shu4"),
            ("数字", "shu4"),
            ("数量", "shu4"),
            ("数数", "shu3"),
            ("数落", "shu3"),
            ("数据", "shu4"),
        ]);
        
        m.insert('乐', vec![
            ("快乐", "le4"),
            ("音乐", "yue4"),
            ("乐器", "yue4"),
            ("乐观", "le4"),
            ("娱乐", "le4"),
        ]);
        
        m.insert('觉', vec![
            ("感觉", "jue2"),
            ("觉得", "jue2"),
            ("睡觉", "jiao4"),
            ("午觉", "jiao4"),
        ]);
        
        m.insert('得', vec![
            ("得到", "de2"),
            ("觉得", "de"),
            ("舍得", "de"),
            ("记得", "de"),
            ("懂得", "de"),
            ("取得", "de2"),
        ]);
        
        m.insert('了', vec![
            ("了解", "liao3"),
            ("明了", "liao3"),
            ("了不起", "liao3"),
            ("来了", "le"),
            ("走了", "le"),
        ]);
        
        m.insert('地', vec![
            ("地方", "di4"),
            ("土地", "di4"),
            ("地址", "di4"),
            ("高兴地", "de"),
            ("慢慢地", "de"),
        ]);
        
        m.insert('还', vec![
            ("还是", "hai2"),
            ("还有", "hai2"),
            ("归还", "huan2"),
            ("还书", "huan2"),
            ("还款", "huan2"),
        ]);
        
        m.insert('都', vec![
            ("都是", "dou1"),
            ("都有", "dou1"),
            ("首都", "du1"),
            ("都市", "du1"),
        ]);
        
        m.insert('为', vec![
            ("为了", "wei4"),
            ("因为", "wei4"),
            ("行为", "wei2"),
            ("认为", "wei2"),
            ("以为", "wei2"),
        ]);
        
        m.insert('没', vec![
            ("没有", "mei2"),
            ("没事", "mei2"),
            ("淹没", "mo4"),
            ("沉没", "mo4"),
        ]);
        
        m.insert('少', vec![
            ("多少", "shao3"),
            ("少年", "shao4"),
            ("少女", "shao4"),
            ("减少", "shao3"),
        ]);
        
        m.insert('种', vec![
            ("种类", "zhong3"),
            ("种子", "zhong3"),
            ("种地", "zhong4"),
            ("种植", "zhong4"),
        ]);
        
        m.insert('着', vec![
            ("看着", "zhe"),
            ("听着", "zhe"),
            ("着急", "zhao2"),
            ("着火", "zhao2"),
        ]);
        
        m.insert('发', vec![
            ("发现", "fa1"),
            ("发生", "fa1"),
            ("头发", "fa4"),
            ("理发", "fa4"),
        ]);
        
        m.insert('会', vec![
            ("会议", "hui4"),
            ("机会", "hui4"),
            ("会计", "kuai4"),
        ]);
        
        m.insert('要', vec![
            ("要求", "yao1"),
            ("重要", "yao4"),
            ("需要", "yao4"),
        ]);
        
        m.insert('当', vec![
            ("当时", "dang1"),
            ("当然", "dang1"),
            ("恰当", "dang4"),
            ("上当", "dang4"),
        ]);
        
        m.insert('处', vec![
            ("地方", "chu4"),
            ("处理", "chu3"),
            ("相处", "chu3"),
            ("到处", "chu4"),
        ]);
        
        m.insert('分', vec![
            ("分开", "fen1"),
            ("分析", "fen1"),
            ("部分", "fen4"),
            ("成分", "fen4"),
        ]);
        
        m.insert('看', vec![
            ("看见", "kan4"),
            ("看书", "kan4"),
            ("看守", "kan1"),
            ("看护", "kan1"),
        ]);
        
        m.insert('好', vec![
            ("好人", "hao3"),
            ("好看", "hao3"),
            ("爱好", "hao4"),
            ("好学", "hao4"),
        ]);
        
        m.insert('中', vec![
            ("中国", "zhong1"),
            ("中心", "zhong1"),
            ("中奖", "zhong4"),
            ("命中", "zhong4"),
        ]);
        
        m.insert('大', vec![
            ("大小", "da4"),
            ("大人", "da4"),
            ("大夫", "dai4"),
        ]);
        
        m.insert('难', vec![
            ("困难", "nan2"),
            ("难过", "nan2"),
            ("灾难", "nan4"),
        ]);
        
        m.insert('和', vec![
            ("和平", "he2"),
            ("和谐", "he2"),
            ("和面", "huo2"),
            ("和药", "huo4"),
        ]);
        
        m.insert('参', vec![
            ("参加", "can1"),
            ("参观", "can1"),
            ("人参", "shen1"),
        ]);
        
        m.insert('差', vec![
            ("差别", "cha1"),
            ("差不多", "cha4"),
            ("出差", "chai1"),
        ]);
        
        m.insert('落', vec![
            ("落下", "luo4"),
            ("落叶", "luo4"),
            ("丢三落四", "la4"),
            ("落枕", "lao4"),
        ]);
        
        m.insert('背', vec![
            ("背后", "bei4"),
            ("背景", "bei4"),
            ("背包", "bei1"),
            ("背负", "bei1"),
        ]);
        
        m.insert('调', vec![
            ("调查", "diao4"),
            ("调整", "tiao2"),
            ("调节", "tiao2"),
        ]);
        
        m.insert('干', vec![
            ("干净", "gan1"),
            ("干燥", "gan1"),
            ("干活", "gan4"),
            ("干部", "gan4"),
        ]);
        
        m.insert('相', vec![
            ("相信", "xiang1"),
            ("相同", "xiang1"),
            ("相片", "xiang4"),
            ("照相", "xiang4"),
        ]);
        
        m.insert('结', vec![
            ("结果", "jie2"),
            ("结束", "jie2"),
            ("结实", "jie1"),
        ]);
        
        m.insert('传', vec![
            ("传说", "chuan2"),
            ("传递", "chuan2"),
            ("传记", "zhuan4"),
            ("水浒传", "zhuan4"),
        ]);
        
        m.insert('角', vec![
            ("角度", "jiao3"),
            ("三角", "jiao3"),
            ("角色", "jue2"),
            ("主角", "jue2"),
        ]);
        
        m.insert('假', vec![
            ("真假", "jia3"),
            ("假如", "jia3"),
            ("放假", "jia4"),
            ("假期", "jia4"),
        ]);
        
        m.insert('省', vec![
            ("省份", "sheng3"),
            ("节省", "sheng3"),
            ("反省", "xing3"),
        ]);
        
        m.insert('倒', vec![
            ("倒水", "dao4"),
            ("倒下", "dao3"),
            ("倒影", "dao4"),
        ]);
        
        m.insert('载', vec![
            ("记载", "zai3"),
            ("载体", "zai4"),
            ("下载", "zai4"),
        ]);
        
        m.insert('教', vec![
            ("教师", "jiao1"),
            ("教育", "jiao4"),
            ("教书", "jiao1"),
        ]);
        
        m.insert('空', vec![
            ("天空", "kong1"),
            ("空气", "kong1"),
            ("空闲", "kong4"),
            ("空白", "kong4"),
        ]);
        
        m.insert('便', vec![
            ("方便", "bian4"),
            ("便利", "bian4"),
            ("便宜", "pian2"),
        ]);
        
        m.insert('似', vec![
            ("相似", "si4"),
            ("似乎", "si4"),
            ("似的", "shi4"),
        ]);
        
        m.insert('降', vec![
            ("降落", "jiang4"),
            ("降低", "jiang4"),
            ("投降", "xiang2"),
        ]);
        
        m
    };
    
    static ref DEFAULT_PRONUNCIATIONS: HashMap<char, &'static str> = {
        let mut m = HashMap::new();
        m.insert('行', "xing2");
        m.insert('重', "zhong4");
        m.insert('长', "chang2");
        m.insert('数', "shu4");
        m.insert('乐', "le4");
        m.insert('觉', "jue2");
        m.insert('得', "de2");
        m.insert('了', "le");
        m.insert('地', "di4");
        m.insert('还', "hai2");
        m.insert('都', "dou1");
        m.insert('为', "wei4");
        m.insert('没', "mei2");
        m.insert('少', "shao3");
        m.insert('种', "zhong3");
        m.insert('着', "zhe");
        m.insert('发', "fa1");
        m.insert('会', "hui4");
        m.insert('要', "yao4");
        m.insert('当', "dang1");
        m.insert('处', "chu4");
        m.insert('分', "fen1");
        m.insert('看', "kan4");
        m.insert('好', "hao3");
        m.insert('中', "zhong1");
        m.insert('大', "da4");
        m.insert('难', "nan2");
        m.insert('和', "he2");
        m.insert('参', "can1");
        m.insert('差', "cha1");
        m.insert('落', "luo4");
        m.insert('背', "bei4");
        m.insert('调', "diao4");
        m.insert('干', "gan1");
        m.insert('相', "xiang1");
        m.insert('结', "jie2");
        m.insert('传', "chuan2");
        m.insert('角', "jiao3");
        m.insert('假', "jia3");
        m.insert('省', "sheng3");
        m.insert('倒', "dao4");
        m.insert('载', "zai3");
        m.insert('教', "jiao1");
        m.insert('空', "kong1");
        m.insert('便', "bian4");
        m.insert('似', "si4");
        m.insert('降', "jiang4");
        m
    };
}

#[derive(Clone)]
pub struct PolyphonicDisambiguator;

impl PolyphonicDisambiguator {
    pub fn new() -> Self {
        Self
    }
    
    pub fn is_polyphonic(&self, c: char) -> bool {
        POLYPHONIC_DICT.contains_key(&c)
    }
    
    pub fn disambiguate(&self, sentence: &str, char_idx: usize) -> Option<String> {
        let chars: Vec<char> = sentence.chars().collect();
        
        if char_idx >= chars.len() {
            return None;
        }
        
        let target_char = chars[char_idx];
        
        if let Some(rules) = POLYPHONIC_DICT.get(&target_char) {
            for (pattern, pronunciation) in rules {
                if self.match_pattern(&chars, char_idx, pattern) {
                    return Some(pronunciation.to_string());
                }
            }
            
            return DEFAULT_PRONUNCIATIONS.get(&target_char)
                .map(|s| s.to_string());
        }
        
        None
    }
    
    fn match_pattern(&self, chars: &[char], char_idx: usize, pattern: &str) -> bool {
        let pattern_chars: Vec<char> = pattern.chars().collect();
        
        if pattern_chars.len() == 2 {
            if char_idx > 0 && chars[char_idx - 1] == pattern_chars[0] {
                return true;
            }
            if char_idx + 1 < chars.len() && chars[char_idx + 1] == pattern_chars[1] {
                return true;
            }
        } else if pattern_chars.len() == 3 {
            if char_idx > 0 && char_idx + 1 < chars.len() {
                if chars[char_idx - 1] == pattern_chars[0] && chars[char_idx + 1] == pattern_chars[2] {
                    return true;
                }
            }
        }
        
        false
    }
    
    pub fn disambiguate_sentence(&self, sentence: &str) -> Vec<(usize, String)> {
        let chars: Vec<char> = sentence.chars().collect();
        let mut results = Vec::new();
        
        for (idx, &ch) in chars.iter().enumerate() {
            if self.is_polyphonic(ch) {
                if let Some(pronunciation) = self.disambiguate(sentence, idx) {
                    results.push((idx, pronunciation));
                }
            }
        }
        
        results
    }
}

impl Default for PolyphonicDisambiguator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_polyphonic_detection() {
        let disambiguator = PolyphonicDisambiguator::new();
        
        assert!(disambiguator.is_polyphonic('行'));
        assert!(disambiguator.is_polyphonic('重'));
        assert!(!disambiguator.is_polyphonic('你'));
    }
    
    #[test]
    fn test_disambiguation() {
        let disambiguator = PolyphonicDisambiguator::new();
        
        let result = disambiguator.disambiguate("银行", 1);
        assert_eq!(result, Some("hang2".to_string()));
        
        let result = disambiguator.disambiguate("行走", 0);
        assert_eq!(result, Some("xing2".to_string()));
        
        let result = disambiguator.disambiguate("重要", 0);
        assert_eq!(result, Some("zhong4".to_string()));
        
        let result = disambiguator.disambiguate("重复", 0);
        assert_eq!(result, Some("chong2".to_string()));
    }
    
    #[test]
    fn test_sentence_disambiguation() {
        let disambiguator = PolyphonicDisambiguator::new();
        
        let results = disambiguator.disambiguate_sentence("银行里有很多重要的人");
        assert!(!results.is_empty());
        
        println!("Disambiguation results: {:?}", results);
    }
}

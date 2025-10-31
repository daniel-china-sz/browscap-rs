use crate::filter::{self, Filter, FilterType};
use crate::literal::Literal;
use crate::rule::Rule;
use crate::{Capabilities, UserAgentParser, literal};
use log::debug;
use std::time::Instant;

pub const COMMON: [&str; 75] = [
    "-",
    "0",
    "1",
    "2",
    "3",
    "4",
    "5",
    "6",
    "7",
    "8",
    "9",
    "profile",
    "player",
    "compatible",
    "android",
    "google",
    "tab",
    "transformer",
    "lenovo",
    "micro",
    "edge",
    "safari",
    "opera",
    "chrome",
    "firefox",
    "msie",
    "chromium",
    "cpu os ",
    "cpu iphone os ",
    "windows nt ",
    "mac os x ",
    "linux",
    "bsd",
    "windows phone",
    "iphone",
    "pad",
    "blackberry",
    "nokia",
    "alcatel",
    "ucbrowser",
    "mobile",
    "ie",
    "mercury",
    "samsung",
    "browser",
    "wow64",
    "silk",
    "lunascape",
    "crios",
    "epiphany",
    "konqueror",
    "version",
    "rv:",
    "build",
    "bot",
    "like gecko",
    "applewebkit",
    "trident",
    "mozilla",
    "windows nt 4",
    "windows nt 5.0",
    "windows nt 5.1",
    "windows nt 5.2",
    "windows nt 6.0",
    "windows nt 6.1",
    "windows nt 6.2",
    "windows nt 6.3",
    "windows nt 10.0",
    "android?4.0",
    "android?4.1",
    "android?4.2",
    "android?4.3",
    "android?4.4",
    "android?2.3",
    "android?5",
];

const FILTER_PREFIXES: [&str; 2] = ["mozilla/5.0", "mozilla/4"];

impl  UserAgentParser {
    pub fn new(mut rules: Vec<Rule>) -> UserAgentParser {
        let timer=Instant::now();
        get_ordered_rules(&mut rules);
        let time=timer.elapsed();
        debug!("order rules time:{:?}",time);
        let my_filters = build_filters(&rules);
        UserAgentParser {
            my_rules: rules,
            my_filters: my_filters,
        }
    }

    pub fn parse(&self, user_agent: &str) -> &Capabilities {
        if user_agent.is_empty() {
            return crate::capabilities::DEFAULT_CAPABILITIES.get().unwrap();
        };
        let mut search_string = literal::get_searchable_string(user_agent.to_lowercase());
        let includes = filter::filter(&mut search_string, &self.my_filters, self.my_rules.len());
        for i in includes.iter_ones() {
            let rule = &self.my_rules[i];
            if rule.matches(&mut search_string) {
                return rule.get_capabilities();
            }
        }
        return crate::capabilities::DEFAULT_CAPABILITIES.get().unwrap();
    }
}

fn build_filters(my_rules: &Vec<Rule>) -> Vec<Filter> {
    let timer=Instant::now();
    let mut result = Vec::new();

    for pattern in FILTER_PREFIXES {
        let literal = Literal::create_literal(pattern);
        let mask = filter::create_prefix_masker(my_rules, pattern);
        result.push(Filter::new(FilterType::Prefix(literal), mask));
    }
    // Build filters for specific contains constraints
    for common in COMMON {
        let literal = Literal::create_literal(common);
        let mask = filter::create_contains_masker(my_rules, common);
        result.push(Filter::new(FilterType::Contains(literal), mask));
    }
    let time=timer.elapsed();
    debug!("build filters time:{:?}",time);
    result
}
fn get_ordered_rules(rules: &mut Vec<Rule>) {
     let total_timer = Instant::now();
    let mut pattern_build_count = 0;
    
    // 阶段1：快速按size排序
    rules.sort_by_key(|r| std::cmp::Reverse(r.get_size()));
    
    // 阶段2：只处理size相同的组
    let mut i = 0;
    while i < rules.len() {
        let current_size = rules[i].get_size();
        let mut j = i + 1;
        
        while j < rules.len() && rules[j].get_size() == current_size {
            j += 1;
        }
        
        if j - i > 1 {
            // 关键优化：组内预计算Pattern
            let patterns: Vec<String> = rules[i..j]
                .iter()
                .map(|r| {
                    pattern_build_count += 1;
                    r.get_pattern()
                })
                .collect();
            
            // 使用预计算的patterns进行排序，但记录原始索引
            let mut indexed_patterns: Vec<(usize, &String)> = 
                patterns.iter().enumerate().collect();
            
            indexed_patterns.sort_by(|a, b| a.1.cmp(b.1));
            
            // 获取排序后的索引顺序
            let mut sorted_indices: Vec<usize> = 
                indexed_patterns.into_iter().map(|(idx, _)| idx).collect();
            
            // 原地重新排列rules（使用交换）
            for pos in 0..(j - i) {
                let current = pos;
                while sorted_indices[current] != current {
                    let target = sorted_indices[current];
                    rules.swap(i + current, i + target);
                    sorted_indices.swap(current, target);
                }
            }
        }
        
        i = j;
    }
    
    debug!("总时间: {:?}", total_timer.elapsed());
    debug!("最终Pattern构建次数: {}", pattern_build_count);
}

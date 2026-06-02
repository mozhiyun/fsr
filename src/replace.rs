//! 模板占位符 → 新项目名的替换规则。

use crate::template::{
    TEMPLATE_API_TITLE, TEMPLATE_SLUG, TEMPLATE_SLUG_SNAKE, TEMPLATE_TITLE,
};

pub fn project_replacements(project_name: &str) -> Vec<(String, String)> {
    let snake = kebab_to_snake(project_name);
    let title = kebab_to_title(project_name);

    let mut pairs = vec![
        (
            TEMPLATE_API_TITLE.to_string(),
            format!("{title} API"),
        ),
        (TEMPLATE_SLUG.to_string(), project_name.to_string()),
        (TEMPLATE_SLUG_SNAKE.to_string(), snake),
        (TEMPLATE_TITLE.to_string(), title),
    ];

    pairs.retain(|(from, to)| from != to);
    pairs.sort_by(|a, b| b.0.len().cmp(&a.0.len()));
    pairs
}

pub fn kebab_to_snake(kebab: &str) -> String {
    kebab.replace('-', "_")
}

pub fn kebab_to_title(kebab: &str) -> String {
    kebab
        .split('-')
        .map(|w| {
            let mut c = w.chars();
            match c.next() {
                None => String::new(),
                Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

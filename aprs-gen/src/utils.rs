// TODO: filtering like this could lead to different player names
//       to end up as the same file name
pub fn file_safe_name(name: &str) -> String {
    name.chars()
        .filter(|&c| !r#"<>:"/\|?*"#.contains(c))
        .collect()
}

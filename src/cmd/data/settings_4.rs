pub static DATA: &'static str = r##"#TUTORIAL=true
[settings]
# There are two variables that can be used anywhere:
# - {cwd}: the path to the directory of the file
# - {repo}: the path to the current repository, which is the closest
#    directory that contains a ".rsk" folder
artifact_paths = ['{repo}/docs']
code_paths = ['{repo}/flash']
exclude_code_paths = []
"##;

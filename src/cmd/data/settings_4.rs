pub static DATA: &'static str = r##"#TUTORIAL=true
[settings]
# There are two variables that can be used anywhere:
# - {cwd}: the path to the directory of the file
# - {repo}: the path to the current repository, which is the closest
#    directory that contains a ".rst" folder
artifact_paths = ['{repo}/reqs']
code_paths = ['{repo}/flash']
exclude_code_paths = []
"##;

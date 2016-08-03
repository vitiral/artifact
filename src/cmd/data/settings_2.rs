pub static DATA: &'static str = r##"#TUTORIAL=true
[settings]
# Any *.rsk file can define additional paths to be loaded by listing them
# in a path.
# There are two variables that can be used anywhere:
# - {cwd}: the path to the directory of the file
# - {repo}: the path to the current repository, which is the closest
#    directory that contains a ".rsk" folder
artifact_paths = ['{repo}/reqs']
code_paths = []
exclude_code_paths = []
"##;

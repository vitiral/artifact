pub static data: &'static str = r##"#TUTORIAL=true
# This is an artifacts file for the command line tool rsk
# files (like this one) that are in {repo}/.rsk are automatically loaded

[settings]
    # Any *.rsk file can define additional paths to be loaded by listing them
# in a path.
# There are two variables that can be used anywhere:
# - {cwd}: the path to the directory of the file
# - {repo}: the path to the current repository, which is the closest
#    directory that contains a ".rsk" folder
## TODO: uncomment this line and define your own paths
artifact_paths = ['{repo}/docs']
"##;

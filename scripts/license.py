import argparse

SOURCE = 'source'
OLD_HEADER = 'old_header'
NEW_HEADER = 'new_header'

parser = argparse.ArgumentParser()
parser.add_argument(SOURCE)
parser.add_argument(OLD_HEADER)
parser.add_argument(NEW_HEADER)

args = parser.parse_args()

with open(args.source) as f:
    source = f.read()

with open(args.old_header) as f:
    old_header = f.read()

with open(args.new_header) as f:
    new_header = f.read()

with open(args.source, 'w') as f:
    f.truncate(0)
    f.write(source.replace(old_header, new_header))

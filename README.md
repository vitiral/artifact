# rsk: the requirements tracking tool made for developers
**rsk** is a intended to be an easy to use tool for a someone at any skill
level or quality background to easily write and track their requirements.

Requirements and design documentation are probably the most important components of
writing quality software: even more important than unit testing or revision control.
Without writing out your requirements and tracking your design specifications, it can
be very difficult to develop and maintain the product you were aiming to create.
However, there are no open source tools (or proprietary tools for that matter) 
that make this process simple, easy and fun. **rsk** aims to do that by giving you a:

 1. simple text-based format to write your requirements in (TOML). This makes it
      easy to track your requirements with the rest of your project using standard
      revision control tools (git, hg, etc)
 2. workflow that is easy for developers to integrate with
 3. ui that is familar and useful -- helping the developer track their own progress
      and maintain requirements -> design -> test documentation.

It is hard to keep documentation up to date, especially when it doesn't aid
the core developer in tracking their progress. **rsk** aims to bridge that gap,
giving you a simple tool that you can use to track your project completeness and
test coverage.

If you like **rsk**, please leave a **star :star:** at github.com/vitiral/rsk. If you find
bugs or have any suggestions, please open an issue there under 
[issues](https://github.com/vitiral/rsk/issues)

## Installation

### With rust and cargo
If you have cargo installed, simply execute

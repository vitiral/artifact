FROM node:7-slim

# Install Elm-lang
RUN npm install -gq elm@0.18.0 elm-test
ENV ELM_HOME /usr/local/lib/node_modules/elm/share

# Get around the locale problem (issue #33) https://github.com/elm-lang/elm-make/issues/33
RUN apt-get -q update && apt-get install -qy locales git
RUN dpkg-reconfigure locales && locale-gen C.UTF-8 && update-locale LANG=C.UTF-8
ENV LC_ALL C.UTF-8

# User management
# Make sure we switch to the correct user/group before doing anything else
ARG user
RUN adduser --disabled-login --gecos "" --uid 1000 --home /home/elm $user
# RUN echo 'export PS1="\W\$ "' >> /home/lfe/.bashrc

USER $user
WORKDIR /home/elm
ENV USER $user
EXPOSE 8000
CMD ["bash"]

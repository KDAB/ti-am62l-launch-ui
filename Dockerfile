# TODO -> could make sense to use a base img closer to the target OS, not sure what they intend to use
FROM rust:slim-bookworm

# TODO -> tidy this up
RUN apt-get update --quiet && \
    apt-get install --assume-yes \
    # docker specific pkgs
    wget \
    # slint specific pkgs -> TODO -> we may want to use another backend instead of Qt?
    clang \
    libfontconfig1-dev \
    libudev-dev \
    qt6-base-dev \
    # slint / skia specific pkgs
    libgbm-dev \
    libinput-dev \
    libxkbcommon-dev

# RUN apt-get update && \
#     DEBIAN_FRONTEND=noninteractive apt-get install --assume-yes \
#     # docker specific pkgs
#     wget \
#     # slint specific pkgs
#     libfontconfig1-dev libxcb1-dev libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev libxkbcommon-dev libinput-dev libgbm-dev \
#     python3 python3-pip python3-launchpadlib \
#     libfontconfig1-dev \
#     curl \
#     clang
#     #libstdc++-10-dev

# Add Docker's official GPG key, add to apt sources and install Docker client CLI
# RUN apt-get install --assume-yes ca-certificates curl && \
#     install -m 0755 -d /etc/apt/keyrings && \
#     curl -fsSL https://download.docker.com/linux/debian/gpg -o /etc/apt/keyrings/docker.asc && \
#     chmod a+r /etc/apt/keyrings/docker.asc && \
#     echo \
#       "deb [arch=$(dpkg --print-architecture) signed-by=/etc/apt/keyrings/docker.asc] https://download.docker.com/linux/debian \
#       $(. /etc/os-release && echo "$VERSION_CODENAME") stable" | \
#       tee /etc/apt/sources.list.d/docker.list > /dev/null && \
#     apt-get update --quiet && \
#     apt-get install docker-ce-cli
RUN wget -O cli.deb https://download.docker.com/linux/debian/dists/bookworm/pool/stable/amd64/docker-ce-cli_26.1.4-1~debian.12~bookworm_amd64.deb
RUN wget -O compose.deb https://download.docker.com/linux/debian/dists/bookworm/pool/stable/amd64/docker-compose-plugin_2.27.1-1~debian.12~bookworm_amd64.deb
RUN apt-get install --assume-yes ./cli.deb ./compose.deb
RUN rm -f ./cli.deb ./compose.deb

RUN rustup component add rustfmt

# ENV CROSS_CONTAINER_IN_CONTAINER=true
RUN cargo install cross
# TODO -> keep this test?
# RUN cross test --no-default-features --aarch64-unknown-linux-gnu

# Work around the Skia source build requiring a newer git version (that supports --path-format=relative with rev-parse, as needed by git-sync-deps.py),
# as well as a disabling of the directory safety checks (https://github.blog/2022-04-12-git-security-vulnerability-announced/#cve-2022-24765) as
# /cargo comes from ~/.cargo and may have differing user ids, which breaks when the skia-bindings build clones additional git repos (skia/third_party/external/*)
# RUN DEBIAN_FRONTEND=noninteractive apt-get install --assume-yes software-properties-common && \
#     add-apt-repository -y ppa:git-core/ppa && \
#     apt-get install --assume-yes git && \
#     git config --global safe.directory '*'

ENV SLINT_STYLE=fluent

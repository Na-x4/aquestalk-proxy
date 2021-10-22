FROM i386/debian as wine

RUN INSTALL_DEPS=' \
  curl \
  ca-certificates \
  gnupg2 \
  ' \
  && apt-get update \
  && apt-get install --no-install-recommends -y $INSTALL_DEPS \
  && curl https://dl.winehq.org/wine-builds/winehq.key -sSf | gpg --dearmor > /usr/share/keyrings/winehq.gpg \
  && . /etc/os-release \
  && echo "deb [signed-by=/usr/share/keyrings/winehq.gpg] https://dl.winehq.org/wine-builds/debian/ $VERSION_CODENAME main" > /etc/apt/sources.list.d/wine-builds.list \
  && apt-get update \
  && apt-get install --no-install-recommends -y \
  winehq-stable \
  && apt-get purge -y --auto-remove $INSTALL_DEPS \
  && apt-get clean \
  && rm -rf /var/lib/apt/lists/*
ENV WINEDEBUG=-all

RUN useradd -m user
USER user
WORKDIR /home/user

FROM rust as builder

RUN apt-get update \
  && apt-get install --no-install-recommends -y \
  mingw-w64 \
  unzip \
  && rm -rf /var/lib/apt/lists/* \
  && rustup target add i686-pc-windows-gnu

RUN useradd -m user
USER user
WORKDIR /home/user
COPY --chown=user:user / /home/user/aquestalk-proxy
RUN cd aquestalk-proxy \
  && cargo build --release --target i686-pc-windows-gnu \
  && unzip aqtk_mv_20090609.zip \
  && mkdir app && cd app \
  && mv ../target/i686-pc-windows-gnu/release/aquestalk-proxy.exe ./aquestalk-proxy.exe \
  && mv ../AquesTalk_mv/bin/ ./bin/

FROM wine
COPY --from=builder /home/user/aquestalk-proxy/app /home/user/app

CMD ["wine", "/home/user/app/aquestalk-proxy.exe"]

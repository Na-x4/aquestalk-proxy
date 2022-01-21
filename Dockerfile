FROM i386/debian as wine

ENV WINEDEBUG=-all

RUN INSTALL_DEPS=' \
  ca-certificates \
  curl \
  gnupg2 \
  ' \
  && export DEBIAN_FRONTEND=noninteractive \
  && apt-get update \
  && apt-get install --no-install-recommends -y $INSTALL_DEPS \
  gosu \
  tini \
  && curl https://dl.winehq.org/wine-builds/winehq.key -sSf | gpg --dearmor > /usr/share/keyrings/winehq.gpg \
  && . /etc/os-release \
  && echo "deb [signed-by=/usr/share/keyrings/winehq.gpg] https://dl.winehq.org/wine-builds/debian/ $VERSION_CODENAME main" > /etc/apt/sources.list.d/wine-builds.list \
  && apt-get update \
  && apt-get install --no-install-recommends -y \
  winehq-stable \
  && apt-get purge -y --auto-remove $INSTALL_DEPS \
  && apt-get clean \
  && rm -rf /var/lib/apt/lists/* \
  && useradd -m user \
  && gosu user wine cmd.exe /c echo. > /dev/null 2>&1

FROM rust as builder

RUN export DEBIAN_FRONTEND=noninteractive \
  && apt-get update \
  && apt-get install --no-install-recommends -y \
  mingw-w64 \
  unzip \
  && apt-get clean \
  && rm -rf /var/lib/apt/lists/* \
  && rustup target add i686-pc-windows-gnu

RUN useradd -m user
USER user
WORKDIR /home/user
COPY --chown=user:user / /home/user/aquestalk-proxy
RUN cd aquestalk-proxy \
  && cargo build --release \
  && mkdir app && cd app \
  && mv ../target/i686-pc-windows-gnu/release/aquestalk-proxy.exe ./aquestalk-proxy.exe \
  && ../scripts/extract-aqtk.sh \
  && cp ../README.md ./ \
  && cp ../COPYING ./

FROM wine
COPY --from=builder --chown=root:root /home/user/aquestalk-proxy/app /app

USER user
EXPOSE 21569
ENTRYPOINT ["/usr/bin/tini", "--", "/usr/bin/wine", "/app/aquestalk-proxy.exe"]
CMD ["--path=/app/aquestalk", "--listen=0.0.0.0:21569"]

# (c) Copyright 2019-2023 MIT
FROM rust:1.66.0-alpine3.17

ENV VIPS_VERSION=8.14.2

RUN apk add --update --no-cache --repository=https://dl-cdn.alpinelinux.org/alpine/v3.17/main \
            build-base \
            pkgconfig \
            clang \
            clang-libclang \
            cgif-dev \
            expat-dev \
            fftw-dev \
            giflib-dev \
            glib-dev \
            gobject-introspection-dev \
            lcms2-dev \
            libexif-dev \
            libgsf-dev \
            libheif-dev \
            libimagequant-dev \
            libjpeg-turbo-dev \
            libpng-dev \
            librsvg-dev \
            libwebp-dev \
            openssl-dev \
            orc-dev \
            pango-dev \
            tiff-dev \
            meson \
            ninja

RUN wget https://github.com/libvips/libvips/releases/download/v${VIPS_VERSION}/vips-${VIPS_VERSION}.tar.xz

RUN mkdir vips

RUN tar xvJf vips-${VIPS_VERSION}.tar.xz -C vips --strip-components 1

WORKDIR /vips

RUN meson setup build --prefix /usr/local/vips
RUN cd build && meson compile
RUN cd build && meson install

RUN ldconfig /etc/ld.so.conf.d

WORKDIR /

RUN rm -rf vips

RUN rm -f vips-${VIPS_VERSION}.tar.xz

RUN rustup component add rustfmt

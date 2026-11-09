QT += core gui qml quick quickcontrols2

CONFIG += c++17 release
TARGET = omatype
TEMPLATE = app

# The Rust engine is built by cargo into a static library and linked in.
# Cargo decides what needs rebuilding, so it runs on every make.
ENGINE_TARGET_DIR = $$PWD/build/engine
ENGINE_LIB = $$ENGINE_TARGET_DIR/release/libomatype_engine.a
engine.target = engine
engine.commands = cargo build --release --locked --manifest-path $$PWD/engine/Cargo.toml --target-dir $$ENGINE_TARGET_DIR
QMAKE_EXTRA_TARGETS += engine
PRE_TARGETDEPS += engine
LIBS += $$ENGINE_LIB -lsqlite3 -ldl -lpthread -lm

HEADERS += \
    engine/include/omatype_engine.h \
    src/backend.h \
    src/cli.h \
    src/engine.h \
    src/theme.h \
    src/translator.h \
    src/wordmodel.h

SOURCES += \
    src/main.cpp \
    src/backend.cpp \
    src/cli.cpp \
    src/engine.cpp \
    src/theme.cpp \
    src/translator.cpp \
    src/wordmodel.cpp

RESOURCES += src/resources.qrc

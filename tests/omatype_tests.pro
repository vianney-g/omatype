QT += core gui quick quickcontrols2 testlib
CONFIG += c++17 testcase
TARGET = omatype_tests
TEMPLATE = app

INCLUDEPATH += ../src

# Links the engine bin/test just built with cargo.
LIBS += $$PWD/../build/engine/release/libomatype_engine.a -lsqlite3 -ldl -lpthread -lm
PRE_TARGETDEPS += $$PWD/../build/engine/release/libomatype_engine.a

HEADERS += ../src/backend.h ../src/engine.h ../src/theme.h ../src/translator.h ../src/wordmodel.h
SOURCES += omatype_tests.cpp ../src/backend.cpp ../src/engine.cpp ../src/theme.cpp ../src/translator.cpp ../src/wordmodel.cpp
RESOURCES += ../src/resources.qrc

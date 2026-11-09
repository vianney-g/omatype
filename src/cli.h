#pragma once

#include <QStringList>

// Commands that need no display, so scripts and agents can read the history:
// `omatype stats`, `omatype history`, `omatype test <id>`, `omatype skill install`.
// Returns -1 when the arguments ask for the window instead.
int runCommand(const QStringList &arguments);

bool isCommand(const QStringList &arguments);

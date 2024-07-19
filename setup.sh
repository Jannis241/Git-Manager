#!/bin/bash

# 1. Erstellen Sie das Release-Build des Projekts
cargo build --release

# 2. Kopieren Sie die ausführbare Datei nach /usr/local/bin und benennen Sie sie in "gm" um (mit sudo)
sudo cp target/release/git-manager /usr/local/bin/gm

# 3. Erstellen Sie das Verzeichnis ~/bin, falls es nicht existiert
mkdir -p ~/bin

# 4. Kopieren Sie die ausführbare Datei in das Verzeichnis ~/bin und benennen Sie sie in "gm" um
cp target/release/git-manager ~/bin/gm

# 5. Sicherstellen, dass ~/bin nur einmal im PATH enthalten ist
if ! grep -q 'export PATH="$HOME/bin:$PATH"' ~/.bashrc; then
    echo 'export PATH="$HOME/bin:$PATH"' >> ~/.bashrc
fi

# 6. Laden Sie die Änderungen in ~/.bashrc
source ~/.bashrc
#!/bin/bash

# Test de la configuration
echo "🛠️ Test de configuration LeKickerFou"

# Test avec un salon vocal fictif
./target/release/lekickerfou --channel 123456789012345678 --config-file test-config.json --help

echo "✅ Tests terminés"

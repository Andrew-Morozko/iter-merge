#!/bin/bash

if grep -q "# fuzzing_cfg_opt" Cargo.toml; then
    # Disable fuzzing configuration
    sed -i '' -r -e 's/^### //' -e 's/^(.*) # fuzzing_cfg_opt$/## \1/' Cargo.toml
else
    # Enable fuzzing configuration
    sed -i '' -r -e '/^(edition|rust-version)/s/^/### /' -e 's/^## (.*)/\1 # fuzzing_cfg_opt/' Cargo.toml
fi

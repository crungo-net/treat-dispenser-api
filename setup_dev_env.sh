#!/usr/bin/env bash
# filepath: /home/vincent/projects/treat-dispenser-api/setup.sh
set -e

echo "🦀 Setting up Treat Dispenser API development environment..."

# Check if running on Raspberry Pi
IS_RASPBERRY_PI=false
if [ -f /etc/os-release ]; then
    . /etc/os-release
    if [[ "$ID" == "raspbian" ]]; then
        IS_RASPBERRY_PI=true
        echo "🥧 Detected Raspberry Pi environment"
    fi
fi

# Install system dependencies
echo "📦 Installing system dependencies..."
if [ "$IS_RASPBERRY_PI" = true ]; then
    sudo apt-get update
    sudo apt-get install -y build-essential pkg-config libssl-dev curl git
else
    # For non-Raspberry Pi environments
    if command -v apt-get &> /dev/null; then
        sudo apt-get update
        sudo apt-get install -y build-essential pkg-config libssl-dev curl git
    elif command -v dnf &> /dev/null; then
        sudo dnf install -y gcc openssl-devel curl git
    elif command -v pacman &> /dev/null; then
        sudo pacman -Sy base-devel openssl curl git
    else
        echo "⚠️ Unsupported package manager. Please install build tools manually."
    fi
fi

# Install Rust if not already installed
if ! command -v rustc &> /dev/null; then
    echo "🦀 Installing Rust..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source "$HOME/.cargo/env"
else
    echo "✅ Rust is already installed"
fi

# Install just command runner
if ! command -v just &> /dev/null; then
    echo "📋 Installing just command runner..."
    cargo install just
else
    echo "✅ just is already installed"
fi

# Install cargo-tarpaulin for code coverage (skip on Raspberry Pi due to compile issues)
if [ "$IS_RASPBERRY_PI" = false ] && ! command -v cargo-tarpaulin &> /dev/null; then
    echo "🧪 Installing cargo-tarpaulin for code coverage..."
    cargo install cargo-tarpaulin
else
    if [ "$IS_RASPBERRY_PI" = true ]; then
        echo "ℹ️ Skipping cargo-tarpaulin installation on Raspberry Pi"
    else
        echo "✅ cargo-tarpaulin is already installed"
    fi
fi

# Install cargo-audit for security auditing
if ! command -v cargo-audit &> /dev/null; then
    echo "🔒 Installing cargo-audit for security checking..."
    cargo install cargo-audit
else
    echo "✅ cargo-audit is already installed"
fi

# Setup just bash completion
echo "🔄 Setting up just command completion..."
mkdir -p ~/.local/share/bash-completion/completions
curl -s https://raw.githubusercontent.com/casey/just/master/completions/just.bash > ~/.local/share/bash-completion/completions/just
chmod +x ~/.local/share/bash-completion/completions/just

# Add to bashrc if not already there
if ! grep -q "just.bash" ~/.bashrc; then
    echo '# Enable just command completion' >> ~/.bashrc
    echo 'if [ -f ~/.local/share/bash-completion/completions/just ]; then' >> ~/.bashrc
    echo '    . ~/.local/share/bash-completion/completions/just' >> ~/.bashrc
    echo 'fi' >> ~/.bashrc
    echo "ℹ️ Added just completion to ~/.bashrc"
else
    echo "✅ just completion already in ~/.bashrc"
fi

# Create .env file if it doesn't exist
if [ ! -f .env ]; then
    echo "📝 Creating .env file..."
    cat > .env << EOF
DISPENSER_API_TOKEN=dev_token_for_local_testing
DISPENSER_API_PORT=3500
RUST_LOG=info
MOTOR_TYPE=StepperMock
EOF
else
    echo "✅ .env file already exists"
fi

# Create .env.test file if it doesn't exist
if [ ! -f .env.test ]; then
    echo "📝 Creating .env.test file..."
    cat > .env.test << EOF
DISPENSER_API_TOKEN=test_token
DISPENSER_API_PORT=0
RUST_LOG=debug
MOTOR_TYPE=StepperMock
EOF
else
    echo "✅ .env.test file already exists"
fi

echo "🎉 Setup complete! To get started:"
echo "1. Run 'source ~/.bashrc' to enable just command completion"
echo "2. Run 'just --list' to see available commands"
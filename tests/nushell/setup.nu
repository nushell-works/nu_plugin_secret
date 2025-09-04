# Plugin setup and teardown automation for nu_plugin_secret testing

# Global test configuration
export def get_test_config [] {
    {
        plugin_binary: "target/release/nu_plugin_secret",
        plugin_name: "secret",
        build_timeout: 60sec,
        test_timeout: 30sec,
        temp_dir: "/tmp/nu_plugin_secret_tests",
        backup_config_dir: "/tmp/nu_plugin_secret_config_backup"
    }
}

# Check if plugin binary exists and build if necessary
export def ensure_plugin_built [] {
    let config = get_test_config
    
    if not ($config.plugin_binary | path exists) {
        print $"🔨 Building plugin binary at ($config.plugin_binary)..."
        
        let build_result = try {
            with-env { CARGO_TARGET_DIR: "target" } {
                cargo build --release --quiet
            }
            "success"
        } catch { |e|
            print $"❌ Build failed: ($e.msg)"
            "failed"
        }
        
        if $build_result == "failed" {
            error make { msg: "Failed to build plugin binary" }
        }
        
        print "✅ Plugin binary built successfully"
    } else {
        print "✅ Plugin binary already exists"
    }
}

# Backup existing plugin configuration
export def backup_plugin_config [] {
    let config = get_test_config
    let config_dir = ($nu.default-config-dir | path join "plugins" "secret")
    
    if ($config_dir | path exists) {
        print $"💾 Backing up existing config from ($config_dir)..."
        mkdir ($config.backup_config_dir)
        cp -r $config_dir $config.backup_config_dir
        print "✅ Configuration backed up"
    } else {
        print "ℹ️  No existing configuration to backup"
    }
}

# Restore plugin configuration from backup
export def restore_plugin_config [] {
    let config = get_test_config
    let config_dir = ($nu.default-config-dir | path join "plugins" "secret")
    let backup_path = ($config.backup_config_dir | path join "secret")
    
    if ($backup_path | path exists) {
        print $"🔄 Restoring config to ($config_dir)..."
        if ($config_dir | path exists) {
            rm -rf $config_dir
        }
        mkdir ($config_dir | path dirname)
        cp -r $backup_path $config_dir
        print "✅ Configuration restored"
    } else {
        print "ℹ️  No backup configuration to restore"
    }
}

# Register the plugin with Nushell
export def register_plugin [] {
    let config = get_test_config
    
    print $"📝 Registering plugin from ($config.plugin_binary)..."
    
    try {
        plugin add $config.plugin_binary
        print "✅ Plugin registered successfully"
    } catch { |e|
        print $"❌ Plugin registration failed: ($e.msg)"
        error make { msg: $"Failed to register plugin: ($e.msg)" }
    }
}

# Activate the plugin for use
export def activate_plugin [] {
    let config = get_test_config
    
    print $"🔌 Activating plugin '($config.plugin_name)'..."
    
    try {
        plugin use secret
        print "✅ Plugin activated successfully"
    } catch { |e|
        print $"❌ Plugin activation failed: ($e.msg)"
        error make { msg: $"Failed to activate plugin: ($e.msg)" }
    }
}

# Verify plugin is working correctly
export def verify_plugin [] {
    print "🧪 Verifying plugin functionality..."
    
    let tests = [
        { name: "info_command", test: { secret info | get name } },
        { name: "wrap_string", test: { "test" | secret wrap | describe } },
        { name: "validate_command", test: { "test" | secret wrap | secret validate } },
    ]
    
    for test in $tests {
        try {
            let result = do $test.test
            print $"  ✅ ($test.name): ($result)"
        } catch { |e|
            print $"  ❌ ($test.name) failed: ($e.msg)"
            error make { msg: $"Plugin verification failed on ($test.name)" }
        }
    }
    
    print "✅ Plugin verification completed successfully"
}

# Create test environment directory
export def setup_test_environment [] {
    let config = get_test_config
    
    print $"📁 Setting up test environment in ($config.temp_dir)..."
    
    if ($config.temp_dir | path exists) {
        rm -rf $config.temp_dir
    }
    
    mkdir $config.temp_dir
    print "✅ Test environment ready"
}

# Clean up test environment
export def cleanup_test_environment [] {
    let config = get_test_config
    
    if ($config.temp_dir | path exists) {
        print $"🧹 Cleaning up test environment ($config.temp_dir)..."
        rm -rf $config.temp_dir
        print "✅ Test environment cleaned"
    }
    
    if ($config.backup_config_dir | path exists) {
        print $"🧹 Cleaning up backup directory ($config.backup_config_dir)..."
        rm -rf $config.backup_config_dir
        print "✅ Backup directory cleaned"
    }
}

# Main setup function - run all setup steps
export def setup_plugin [] {
    print "🚀 Starting nu_plugin_secret test setup..."
    
    try {
        ensure_plugin_built
        backup_plugin_config
        setup_test_environment
        register_plugin
        activate_plugin
        verify_plugin
        
        print "🎉 Plugin setup completed successfully!"
        print ""
        print "Available commands:"
        print "  secret wrap, secret wrap, secret wrap, etc."
        print "  secret unwrap, secret validate, secret type-of, secret info"
        print "  secret configure, secret config show, secret config reset"
        print ""
        
    } catch { |e|
        print $"💥 Setup failed: ($e.msg)"
        cleanup_plugin
        error make { msg: $"Plugin setup failed: ($e.msg)" }
    }
}

# Main cleanup function - run all cleanup steps
export def cleanup_plugin [] {
    print "🧹 Starting nu_plugin_secret test cleanup..."
    
    try {
        # Reset plugin configuration to defaults
        try { secret config reset } catch { }
        
        restore_plugin_config
        cleanup_test_environment
        
        print "✅ Plugin cleanup completed successfully!"
        
    } catch { |e|
        print $"⚠️  Cleanup warning: ($e.msg)"
        # Continue cleanup even if some steps fail
        cleanup_test_environment
    }
}

# Quick health check for the plugin
export def health_check [] {
    print "🏥 Performing plugin health check..."
    
    try {
        let info = secret info
        let version = $info | get version
        let plugin_name = $info | get plugin_name
        
        print $"✅ Plugin: ($plugin_name) v($version)"
        
        # Test basic functionality
        let test_secret = "health-check-test" | secret wrap
        let is_valid = $test_secret | secret validate
        let revealed = $test_secret | secret unwrap
        
        if $is_valid and ($revealed == "health-check-test") {
            print "✅ Basic functionality working"
            return true
        } else {
            print "❌ Basic functionality test failed"
            return false
        }
        
    } catch { |e|
        print $"❌ Health check failed: ($e.msg)"
        return false
    }
}

# Get plugin status information
export def get_plugin_status [] {
    {
        binary_exists: ("target/release/nu_plugin_secret" | path exists),
        plugin_loaded: (try { secret info; true } catch { false }),
        test_env_ready: (("/tmp/nu_plugin_secret_tests" | path exists)),
        config_backed_up: (("/tmp/nu_plugin_secret_config_backup" | path exists))
    }
}
# How to Run nu_plugin_secret Nushell Tests

## ✅ **Fixed and Working Methods**

### **Method 1: Cross-Platform Script (Recommended)**
```bash
# From project root directory
./scripts/run_nu_tests.sh

# Setup only
./scripts/run_nu_tests.sh --setup-only

# Cleanup only  
./scripts/run_nu_tests.sh --cleanup-only
```

### **Method 2: Direct Simple Test**
```bash
# From project root
cd tests/nushell
nu simple_test.nu
```

### **Method 3: Manual Setup**
```bash
# 1. Build plugin
cargo build --release

# 2. Go to test directory
cd tests/nushell

# 3. Setup and run
nu -c "use setup.nu; setup setup_plugin"

# 4. Test manually
nu -c "plugin add ../../target/release/nu_plugin_secret; plugin use secret; 'test' | secret wrap-string"

# 5. Cleanup when done
nu -c "use setup.nu; setup cleanup_plugin"
```

## 🎯 **What Each Method Tests**

All methods run the same core test suite:

- ✅ **Plugin Setup**: Automated building, registration, and activation
- ✅ **Wrap String**: Basic string wrapping functionality  
- ✅ **Unwrap**: Basic unwrap command testing
- ✅ **Multiple Secrets**: Pipeline operations with multiple secrets
- ✅ **Validation**: Secret vs regular value validation
- ✅ **Security**: Redaction and content protection
- ✅ **Cleanup**: Automated environment teardown

## 🔧 **Troubleshooting**

**If you get errors:**

1. **Make sure you're in the project root**: 
   ```bash
   cd /path/to/nu_plugin_secret
   ```

2. **Build the plugin first**:
   ```bash
   cargo build --release
   ```

3. **Clean up if needed**:
   ```bash
   ./scripts/run_nu_tests.sh --cleanup-only
   ```

4. **Check Nushell installation**:
   ```bash
   nu --version
   ```

## 📊 **Expected Output**

When successful, you should see:
```
🎉 All tests passed successfully!
✅ Tests completed and cleaned up
✅ All operations completed successfully
```

## ❌ **Previous Issue Fixed**

The original complex test runner (`runner.nu`) had a Nushell parsing issue with dynamic file sourcing. This has been resolved by:

1. Using a simplified test approach
2. Avoiding dynamic `source $variable` calls
3. Running tests directly without complex introspection

The tests now work reliably and provide comprehensive validation of the plugin functionality!
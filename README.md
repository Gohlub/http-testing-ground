# Todo App with HTTP Handler Testing

A demonstration todo application built with the hyperprocess macro framework, designed to validate and showcase comprehensive HTTP routing capabilities.

## What We're Testing

This project serves as a comprehensive test suite for the hyperprocess macro's HTTP routing system. We're validating that the routing logic correctly handles complex scenarios that previously caused issues:

1. **Handler Priority & Specificity**
   - Ensuring specific path handlers take precedence over catch-all handlers
   - Validating that `#[http(method = "GET", path = "/users")]` beats `#[http(method = "GET")]`

2. **Parameter vs Parameter-less Handler Coexistence** 
   - Testing that handlers with parameters don't get blocked by overly broad parameter-less handlers
   - Ensuring the macro can distinguish between handlers that expect request bodies vs those that don't

3. **Method + Path Uniqueness**
   - Verifying each (HTTP method + path) combination maps to exactly one handler
   - Preventing ambiguous routing where multiple handlers could match the same request

4. **Dynamic Fallback Routing**
   - Testing intelligent fallback logic for unmatched requests
   - Validating path-based routing decisions within fallback handlers

### **Test Scenarios Covered:**

- **Specific Path Handlers**: Exact route matching (`GET /users`, `POST /api/data`)
- **Dynamic Fallback Handlers**: Method-specific catch-alls with path logic
- **Ultimate Catch-All**: Handles any unmatched method/path combination  
- **Error Handling**: Wrong handler names, invalid JSON, missing request bodies

## Setup

1. Install 
   ```bash
   kit bs --hyperapp
   ```

With a running node

## Testing

### Run the Test Suite

Execute the comprehensive HTTP handler test script:

```bash
chmod +x test.sh
./test.sh
```




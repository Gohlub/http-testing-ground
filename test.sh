#!/bin/bash

# Simple HTTP routing test script  
BASE_URL="http://localhost:8080/todo:todo:template.os"

echo "🧪 Testing simplified HTTP routing patterns..."
echo "Base URL: $BASE_URL"
echo ""

# Function to test endpoint with smart response formatting
test_endpoint() {
    local method="$1"
    local path="$2" 
    local data="$3"
    local description="$4"
    
    echo "Testing: $description"
    echo "  $method $BASE_URL$path"
    
    if [ -n "$data" ]; then
        echo "  Data: $data"
        local response=$(curl -s -X "$method" "$BASE_URL$path" \
            -H "Content-Type: application/json" \
            -d "$data")
    else
        local response=$(curl -s -X "$method" "$BASE_URL$path")
    fi
    
    # Check if response looks like HTML
    if [[ "$response" == *"<!DOCTYPE"* ]] || [[ "$response" == *"<html"* ]] || [[ "$response" == *"<HTML"* ]]; then
        echo "  Response: [HTML Document - UI loaded successfully]"
    elif [[ "$response" == *"<"*">"* ]] && [[ ${#response} -gt 200 ]]; then
        echo "  Response: [HTML/XML Content - ${#response} characters]"
    else
        echo "  Response: $response"
    fi
    echo ""
}

echo "=== SPECIFIC PATH HANDLERS ==="

test_endpoint "GET" "/users" "" " GET /users (specific handler)"
test_endpoint "POST" "/users" '{"CreateUser": {"message": "John Doe", "id": 1}}' " POST /users (specific handler with params)"
test_endpoint "GET" "/posts" "" " GET /posts (specific handler)"
test_endpoint "POST" "/api/data" '{"ProcessData": {"message": "test data", "id": 42}}' " POST /api/data (specific handler with params)"

echo "=== DYNAMIC FALLBACK HANDLERS ==="

test_endpoint "GET" "/api/unknown" "" " GET /api/unknown (should hit API GET fallback)"
test_endpoint "GET" "/admin/dashboard" "" " GET /admin/dashboard (should hit admin GET fallback)" 
test_endpoint "GET" "/test/something" "" " GET /test/something (should hit test GET fallback)"

test_endpoint "POST" "/api/upload" '{"HandlePostFallback": {"message": "upload data", "id": 99}}' "🔄 POST /api/upload (should hit POST fallback)"
test_endpoint "POST" "/other/endpoint" '{"HandlePostFallback": {"message": "other data", "id": 88}}' "🔄 POST /other/endpoint (should hit POST fallback)"

echo "=== CATCH-ALL HANDLER ==="

test_endpoint "PUT" "/anything" "" " PUT /anything (should hit catch-all)"
test_endpoint "DELETE" "/whatever" "" " DELETE /whatever (should hit catch-all)"
test_endpoint "PATCH" "/some/path" "" " PATCH /some/path (should hit catch-all)"

echo "=== ERROR CASES ==="

test_endpoint "POST" "/users" '{"WrongHandler": {"message": "test"}}' " POST /users with wrong handler name"
test_endpoint "POST" "/users" 'invalid json' " POST /users with invalid JSON"
test_endpoint "POST" "/users" "" " POST /users with no body"

echo "Test completed!" 

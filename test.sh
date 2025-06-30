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

echo "=== ASYNC LONG-RUNNING TEST ==="
echo "Testing concurrent async request handling..."
echo "  1. Starting slow endpoint (5s delay) in background"
echo "  2. Immediately calling fast endpoint"
echo ""

# Start the slow request in the background
echo "Starting POST /users-slow in background..."
curl -s -X POST "$BASE_URL/users-slow" \
    -H "Content-Type: application/json" \
    -d '{"CreateUserSlow": {"message": "Slow User", "id": 1000}}' > slow_response.txt &
SLOW_PID=$!

# Give it a moment to start
sleep 0.5

# Now call the fast endpoint
echo "Calling POST /users immediately..."
START_TIME=$(date +%s)
FAST_RESPONSE=$(curl -s -X POST "$BASE_URL/users" \
    -H "Content-Type: application/json" \
    -d '{"CreateUser": {"message": "Fast User", "id": 2000}}')
END_TIME=$(date +%s)
FAST_DURATION=$((END_TIME - START_TIME))

echo "  Fast endpoint completed in ${FAST_DURATION}s"
echo "  Response: $FAST_RESPONSE"
echo ""

# Wait for the slow request to complete
echo "Waiting for slow endpoint to complete..."
wait $SLOW_PID
SLOW_RESPONSE=$(cat slow_response.txt)
rm -f slow_response.txt

echo "  Slow endpoint completed"
echo "  Response: $SLOW_RESPONSE"
echo ""

if [ $FAST_DURATION -lt 3 ]; then
    echo "✅ Success: Fast endpoint returned quickly while slow endpoint was running"
else
    echo "❌ Failed: Fast endpoint took too long (${FAST_DURATION}s)"
fi

echo "Test completed!"

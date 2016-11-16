# Manual tests
After making changes all the tests should be run on both connman and Network Manager.

## Test 1
 1. Retry flag = false
 2. No credentials stored
 3. Make sure the hotspot starts
 4. Set correct credentials
 5. Make sure it connects and then exits

## Test 2
 1. Retry flag = false
 2. Correct credentials stored
 3. Make sure it connects and then exits

## Test 3
 1. Retry flag = false
 2. No credentials stored
 3. Make sure the hotspot starts
 4. Set incorrect credentials (password less than 8 chars)
 5. Make sure it does not connect and then exits

## Test 4
 1. Retry flag = false
 2. No credentials stored
 3. Make sure the hotspot starts
 4. Set incorrect credentials (password greater than 8 chars)
 5. Make sure it does not connect and then exits

## Test 5
 1. Retry flag = false
 2. Incorrect credentials stored
 3. Make sure it does not connect and then exits

## Test 6
 1. Retry flag = true
 2. No credentials stored
 3. Make sure the hotspot starts
 4. Set correct credentials
 5. Make sure it connects and then exits

## Test 7
 1. Retry flag = true
 2. Correct credentials stored
 3. Make sure it connects and then exits
 
## Test 8
 1. Retry flag = true
 2. No credentials stored
 3. Make sure the hotspot starts
 4. Set incorrect credentials (password less than 8 chars)
 5. Make sure it does not connect and go to Test 6, Step 3

## Test 9
 1. Retry flag = true
 2. No credentials stored
 3. Make sure the hotspot starts
 4. Set incorrect credentials (password greater than 8 chars)
 5. Make sure it does not connect and go to Test 6, Step 3
 
## Test 10
 1. Retry flag = true
 2. Incorrect credentials stored
 3. Make sure it does not connect and go to Test 6, Step 3


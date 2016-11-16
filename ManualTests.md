# Manual tests
After making changes all the tests should be run on both connman and Network Manager.

### Test 1
 1. No credentials stored
 2. Run start script
 3. Enter incorrect credentials (less than 8 chars)
 4. Make sure it retries
 5. Enter incorrect credentials (more than 8 chars)
 6. Make sure it retries
 7. Enter correct credentials
 8. Make sure it connects and exits

### Test 2
 1. Correct credentials stored
 2. Run start script
 3. Make sure it connects and exits

### Test 3
 1. Incorrect credentials stored
 2. Run start script
 3. Make sure it fails to connect and exits

### Test 4
 1. No credentials stored
 2. Run node app.js
 3. Enter incorrect credentials (less than 8 chars)
 4. Make sure it retries
 5. Enter incorrect credentials (more than 8 chars)
 6. Make sure it retries
 7. Enter correct credentials
 8. Make sure it connects and exits

### Test 5
 1. Correct credentials stored
 2. Run node app.js
 3. Enter incorrect credentials (less than 8 chars)
 4. Make sure it retries
 5. Enter incorrect credentials (more than 8 chars)
 6. Make sure it retries
 7. Enter correct credentials
 8. Make sure it connects and exits

### Test 6
 1. Incorrect credentials stored
 2. Run node app.js
 3. Enter incorrect credentials (less than 8 chars)
 4. Make sure it retries
 5. Enter incorrect credentials (more than 8 chars)
 6. Make sure it retries
 7. Enter correct credentials
 8. Make sure it connects and exits
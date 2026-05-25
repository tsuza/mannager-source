# 1.1.0
## New
- Added a transition to the server creation page.
- Added Day of Defeat: Source as a supported game.

## Fixed
- Made port forwarding non-blocking
  - Before, especially if it couldn't port forward, you'd notice that the terminal did not output
    anything until you got a notification about it failing. Not anymore.

# 1.0.4
## Fixed
- Fix notifications on Windows ( previously they were not happening ).
- Fix the notification on SM install not happening
- Made so you cannot change the hosting mode in the UI if the server is running

# 1.0.3
## Fixed
- Correctly initialize the server list config file if it doesn't exist ( regression ).
  - Now you don't have to re-make the server every time. If the file existed already, this wasn't a problem.

# 1.0.2
## Fixed
- Port forwarding now actually works
- You're now properly notified about your app update, and the pop-up gets closed once you press download
- Small UI fix regarding the version number in the app updater

# 1.0.1
- Some UI changes
  - New animated progress bar
  - Added a scrollable where needed

# 1.0.0
- Initial release!

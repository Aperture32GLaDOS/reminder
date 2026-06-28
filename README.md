# Reminder
A simple rust CLI tool to set a reminder at a certain time
## Usage
- reminder at-time HH:MM:SS - this will play a reminder at the given time, on the day which the program was run.
- reminder from-now HH:MM:SS - this will play a reminder after the given amount of time from the running of the program.
- reminder -n - this will not spawn a daemon, and will instead make the program wait until the time has elapsed (useful for bash scripting, for example implementing a pomodoro timer)

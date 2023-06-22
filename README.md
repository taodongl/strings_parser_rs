# Purpose

Check if there is syntax error in resource file, and report the location of issue.

# Dependency

Because `pest` cannot generate accurate location of issues, `nom` is proper for the task:

+ nom
+ nom_locate



# Strings file schema

https://developer.apple.com/library/archive/documentation/Cocoa/Conceptual/LoadingResources/Strings/Strings.html

Just as in C, some characters must be prefixed with a backslash before you can include them in the string. These characters include double quotation marks, the backslash character itself, and special control characters such as linefeed (\n) and carriage returns (\r).

```
"File \"%@\" cannot be opened" = " ... ";
"Type \"OK\" when done" = " ... ";
```

You can include arbitrary Unicode characters in a value string by specifying \U followed immediately by up to four hexadecimal digits. The four digits denote the entry for the desired Unicode character; for example, the space character is represented by hexadecimal 20 and thus would be \U0020 when specified as a Unicode character.




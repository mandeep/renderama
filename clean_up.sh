#!/bin/zsh

# Find files using Zsh globbing
# (N) prevents "no matches found" errors
files=( (render_|denoised_render_)*.(exr|png)(N) )

# Check if array is empty
if (( ${#files} == 0 )); then
    echo "No render artifacts found."
    return 0 2>/dev/null || exit 0
fi

# Print the list of found files
echo "Found ${#files} file(s):"
for f in "${files[@]}"; { echo "  $f" }

# Confirmation
echo -n "Delete these files? (y/n): "
read -k 1 reply
echo

if [[ "$reply" == [yY] ]]; then
    rm -f "${files[@]}"
    echo "Files deleted."
else
    echo "Operation cancelled."
fi
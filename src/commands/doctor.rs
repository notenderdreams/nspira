/*
1. Check if database exists; create or warn if missing.
2. Verify 'projects' table and required fields exist.
3. Iterate over all tracked projects:
   - Check if project path exists on disk.
   - Check if cache_dir exists (warn if missing).
4. Collect and summarize issues:
   - Missing paths
   - Missing cache directories
   - Last cleaned timestamps
5. Print a concise report to the user.
*/

pub fn run (){
    todo!()
}
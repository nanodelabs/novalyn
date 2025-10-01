/**
 * Example: Generate changelog
 * 
 * This example shows how to generate a changelog from git history
 * using the changelogen npm package.
 */

const { generate } = require('changelogen');

async function main() {
  try {
    console.log('Generating changelog...');
    
    const result = await generate({
      from: 'v0.1.0',  // Starting tag/commit
      to: 'HEAD',      // Ending ref (defaults to HEAD)
      write: false,    // Don't write to file, just return content
    });

    console.log('\n📝 Generated Changelog:\n');
    console.log(result.content);
    console.log(`\n✅ Processed ${result.commits} commits`);
    
    if (result.version) {
      console.log(`📦 Version: ${result.version}`);
    }
  } catch (error) {
    console.error('Error generating changelog:', error.message);
    process.exit(1);
  }
}

main();

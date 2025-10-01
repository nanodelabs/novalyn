/**
 * Example: Show next version
 * 
 * This example shows how to get the next semantic version
 * based on conventional commits.
 */

const { showVersion } = require('changelogen');

async function main() {
  try {
    console.log('Inferring next version...');
    
    const version = await showVersion({
      from: 'v0.1.0',  // Starting tag/commit (optional)
    });

    console.log(`\n📦 Next version: ${version}`);
  } catch (error) {
    console.error('Error inferring version:', error.message);
    process.exit(1);
  }
}

main();

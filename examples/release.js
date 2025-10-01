/**
 * Example: Full release
 * 
 * This example shows how to perform a full release:
 * - Generate changelog
 * - Update CHANGELOG.md
 * - Create git tag
 */

const { release } = require('changelogen');

async function main() {
  try {
    console.log('Starting release process...');
    
    const result = await release({
      dryRun: true,  // Set to false to actually create tag
      yes: true,     // Skip confirmation prompts
    });

    console.log('\n✅ Release complete!');
    console.log(`📦 Version: ${result.previousVersion} → ${result.newVersion}`);
    console.log(`📝 Processed ${result.commits} commits`);
    console.log(`🏷️  Tag created: ${result.tagCreated}`);
    
    console.log('\n📝 Changelog preview:\n');
    console.log(result.content.substring(0, 500) + '...');
  } catch (error) {
    console.error('Error during release:', error.message);
    process.exit(1);
  }
}

main();

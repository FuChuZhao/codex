use anyhow::Context;
use anyhow::Result;
use anyhow::bail;
use serde::Deserialize;
use serde::Serialize;
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

const STORE_DIR: &str = ".codex-notes";
const VERSION: u32 = 1;
const NOTE_STATUSES: &[&str] = &["draft", "open", "blocked", "done", "archived"];
const NOTE_PRIORITIES: &[&str] = &["p0", "p1", "p2", "p3"];
static ID_COUNTER: AtomicU64 = AtomicU64::new(0);

#[derive(Debug, clap::Parser)]
pub struct ConversationCli {
    #[command(subcommand)]
    pub subcommand: ConversationSubcommand,
}

#[derive(Debug, clap::Subcommand)]
pub enum ConversationSubcommand {
    /// Create a conversation record.
    Create(ConversationCreateArgs),
    /// List conversations.
    List(ConversationListArgs),
    /// Show one conversation and its messages.
    Show(ConversationShowArgs),
}

#[derive(Debug, clap::Parser)]
pub struct ConversationCreateArgs {
    #[arg(long = "workspace", default_value = ".")]
    pub workspace: PathBuf,

    #[arg(long = "title")]
    pub title: String,
}

#[derive(Debug, clap::Parser)]
pub struct ConversationListArgs {
    #[arg(long = "workspace", default_value = ".")]
    pub workspace: PathBuf,

    #[arg(long)]
    pub json: bool,
}

#[derive(Debug, clap::Parser)]
pub struct ConversationShowArgs {
    #[arg(long = "workspace", default_value = ".")]
    pub workspace: PathBuf,

    #[arg(long = "id")]
    pub id: String,

    #[arg(long)]
    pub json: bool,
}

#[derive(Debug, clap::Parser)]
pub struct MessageCli {
    #[command(subcommand)]
    pub subcommand: MessageSubcommand,
}

#[derive(Debug, clap::Subcommand)]
pub enum MessageSubcommand {
    /// Add one message to a conversation.
    Add(MessageAddArgs),
}

#[derive(Debug, clap::Parser)]
pub struct MessageAddArgs {
    #[arg(long = "workspace", default_value = ".")]
    pub workspace: PathBuf,

    #[arg(long = "conversation")]
    pub conversation: String,

    #[arg(long = "role")]
    pub role: String,

    #[arg(long = "content")]
    pub content: String,

    #[arg(long = "parent")]
    pub parent: Option<String>,

    #[arg(long)]
    pub json: bool,
}

#[derive(Debug, clap::Parser)]
pub struct NoteCli {
    #[command(subcommand)]
    pub subcommand: NoteSubcommand,
}

#[derive(Debug, clap::Subcommand)]
pub enum NoteSubcommand {
    /// Add a note.
    Add(NoteAddArgs),
    /// Attach a note to one message.
    Annotate(NoteAnnotateArgs),
    /// List notes.
    List(NoteListArgs),
    /// Show one note.
    Show(NoteShowArgs),
    /// Update one note.
    Update(NoteUpdateArgs),
    /// Archive one note.
    Archive(NoteArchiveArgs),
}

#[derive(Debug, clap::Parser)]
pub struct NoteAddArgs {
    #[arg(long = "workspace", default_value = ".")]
    pub workspace: PathBuf,

    #[arg(long = "title")]
    pub title: String,

    #[arg(long = "body")]
    pub body: String,

    #[arg(long = "conversation")]
    pub conversation: Option<String>,

    #[arg(long = "message")]
    pub message: Option<String>,

    #[arg(long = "tag")]
    pub tags: Vec<String>,

    #[arg(long = "status", default_value = "open")]
    pub status: String,

    #[arg(long = "priority", default_value = "p2")]
    pub priority: String,

    #[arg(long = "file")]
    pub related_files: Vec<String>,

    #[arg(long)]
    pub json: bool,
}

#[derive(Debug, clap::Parser)]
pub struct NoteAnnotateArgs {
    #[arg(long = "workspace", default_value = ".")]
    pub workspace: PathBuf,

    #[arg(long = "message")]
    pub message: String,

    #[arg(long = "body")]
    pub body: String,

    #[arg(long = "conversation")]
    pub conversation: Option<String>,

    #[arg(long = "tag")]
    pub tags: Vec<String>,

    #[arg(long)]
    pub json: bool,
}

#[derive(Debug, clap::Parser)]
pub struct NoteListArgs {
    #[arg(long = "workspace", default_value = ".")]
    pub workspace: PathBuf,

    #[arg(long = "status")]
    pub status: Option<String>,

    #[arg(long = "tag")]
    pub tag: Option<String>,

    #[arg(long = "conversation")]
    pub conversation: Option<String>,

    #[arg(long = "repo")]
    pub repo: Option<String>,

    #[arg(long)]
    pub json: bool,
}

#[derive(Debug, clap::Parser)]
pub struct NoteShowArgs {
    #[arg(long = "workspace", default_value = ".")]
    pub workspace: PathBuf,

    #[arg(long = "id")]
    pub id: String,

    #[arg(long)]
    pub json: bool,
}

#[derive(Debug, clap::Parser)]
pub struct NoteUpdateArgs {
    #[arg(long = "workspace", default_value = ".")]
    pub workspace: PathBuf,

    #[arg(long = "id")]
    pub id: String,

    #[arg(long = "title")]
    pub title: Option<String>,

    #[arg(long = "body")]
    pub body: Option<String>,

    #[arg(long = "tag")]
    pub tags: Vec<String>,

    #[arg(long = "status")]
    pub status: Option<String>,

    #[arg(long = "priority")]
    pub priority: Option<String>,

    #[arg(long)]
    pub clear_tags: bool,

    #[arg(long)]
    pub json: bool,
}

#[derive(Debug, clap::Parser)]
pub struct NoteArchiveArgs {
    #[arg(long = "workspace", default_value = ".")]
    pub workspace: PathBuf,

    #[arg(long = "id")]
    pub id: String,

    #[arg(long)]
    pub json: bool,
}

#[derive(Debug, clap::Parser)]
pub struct BranchCli {
    #[command(subcommand)]
    pub subcommand: BranchSubcommand,
}

#[derive(Debug, clap::Subcommand)]
pub enum BranchSubcommand {
    /// Fork a new conversation from a message.
    Fork(BranchForkArgs),
    /// Show conversation branch tree.
    Tree(BranchTreeArgs),
}

#[derive(Debug, clap::Parser)]
pub struct BranchForkArgs {
    #[arg(long = "workspace", default_value = ".")]
    pub workspace: PathBuf,

    #[arg(long = "message")]
    pub message: String,

    #[arg(long = "conversation")]
    pub conversation: Option<String>,

    #[arg(long = "title")]
    pub title: Option<String>,

    #[arg(long)]
    pub json: bool,
}

#[derive(Debug, clap::Parser)]
pub struct BranchTreeArgs {
    #[arg(long = "workspace", default_value = ".")]
    pub workspace: PathBuf,

    #[arg(long = "conversation")]
    pub conversation: String,
}

#[derive(Debug, clap::Parser)]
pub struct SnapshotCli {
    #[command(subcommand)]
    pub subcommand: SnapshotSubcommand,
}

#[derive(Debug, clap::Subcommand)]
pub enum SnapshotSubcommand {
    /// Create one snapshot.
    Create(SnapshotCreateArgs),
    /// List snapshots.
    List(SnapshotListArgs),
    /// Show one snapshot.
    Show(SnapshotShowArgs),
    /// Render one snapshot as resume context text.
    Resume(SnapshotResumeArgs),
}

#[derive(Debug, clap::Parser)]
pub struct SnapshotCreateArgs {
    #[arg(long = "workspace", default_value = ".")]
    pub workspace: PathBuf,

    #[arg(long = "conversation")]
    pub conversation: String,

    #[arg(long = "summary")]
    pub summary: Option<String>,

    #[arg(long = "todo")]
    pub todo: Vec<String>,

    #[arg(long = "risk")]
    pub risk: Vec<String>,

    #[arg(long = "from-latest", default_value_t = false)]
    pub from_latest: bool,

    #[arg(long)]
    pub json: bool,
}

#[derive(Debug, clap::Parser)]
pub struct SnapshotListArgs {
    #[arg(long = "workspace", default_value = ".")]
    pub workspace: PathBuf,

    #[arg(long = "conversation")]
    pub conversation: Option<String>,

    #[arg(long)]
    pub json: bool,
}

#[derive(Debug, clap::Parser)]
pub struct SnapshotShowArgs {
    #[arg(long = "workspace", default_value = ".")]
    pub workspace: PathBuf,

    #[arg(long = "id")]
    pub id: String,

    #[arg(long)]
    pub json: bool,
}

#[derive(Debug, clap::Parser)]
pub struct SnapshotResumeArgs {
    #[arg(long = "workspace", default_value = ".")]
    pub workspace: PathBuf,

    #[arg(long = "snapshot")]
    pub snapshot: String,
}

#[derive(Debug, clap::Parser)]
pub struct SearchArgs {
    #[arg(long = "workspace", default_value = ".")]
    pub workspace: PathBuf,

    pub query: String,

    #[arg(long = "tag")]
    pub tag: Option<String>,

    #[arg(long = "status")]
    pub status: Option<String>,

    #[arg(long = "repo")]
    pub repo: Option<String>,

    #[arg(long = "conversation")]
    pub conversation: Option<String>,

    #[arg(long)]
    pub json: bool,
}

#[derive(Debug, clap::Parser)]
pub struct ExportCli {
    #[command(subcommand)]
    pub subcommand: ExportSubcommand,
}

#[derive(Debug, clap::Subcommand)]
pub enum ExportSubcommand {
    /// Export one conversation (with optional branches).
    Conversation(ExportConversationArgs),
}

#[derive(Debug, clap::Parser)]
pub struct ExportConversationArgs {
    #[arg(long = "workspace", default_value = ".")]
    pub workspace: PathBuf,

    #[arg(long = "id")]
    pub id: String,

    #[arg(long = "format", default_value = "md")]
    pub format: String,

    #[arg(long = "with-branches", default_value_t = false)]
    pub with_branches: bool,

    #[arg(long)]
    pub json: bool,
}

#[derive(Debug, clap::Parser)]
pub struct IndexCli {
    #[command(subcommand)]
    pub subcommand: IndexSubcommand,
}

#[derive(Debug, clap::Subcommand)]
pub enum IndexSubcommand {
    /// Rebuild index.
    Rebuild(IndexRebuildArgs),
}

#[derive(Debug, clap::Parser)]
pub struct IndexRebuildArgs {
    #[arg(long = "workspace", default_value = ".")]
    pub workspace: PathBuf,

    #[arg(long)]
    pub json: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct RepoContext {
    repo_path: String,
    git_branch: Option<String>,
    git_commit: Option<String>,
    related_files: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ConversationRecord {
    id: String,
    title: String,
    created_at: i64,
    updated_at: i64,
    root_message_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct MessageRecord {
    id: String,
    conversation_id: String,
    parent_id: Option<String>,
    role: String,
    content: String,
    created_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct NoteRecord {
    id: String,
    conversation_id: String,
    message_id: Option<String>,
    title: String,
    body: String,
    tags: Vec<String>,
    status: String,
    priority: String,
    created_at: i64,
    updated_at: i64,
    repo_ctx: Option<RepoContext>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct BranchRecord {
    id: String,
    source_conversation_id: String,
    source_message_id: String,
    new_conversation_id: String,
    created_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SnapshotRecord {
    id: String,
    conversation_id: String,
    summary: String,
    todo: Vec<String>,
    risks: Vec<String>,
    repo_ctx: Option<RepoContext>,
    created_at: i64,
}

#[derive(Debug, Clone, Serialize)]
struct SearchResultRow {
    kind: String,
    id: String,
    conversation_id: String,
    title: String,
    snippet: String,
    updated_at: i64,
}

#[derive(Debug, Clone, Serialize)]
struct IndexSummary {
    version: u32,
    generated_at: i64,
    conversations: usize,
    messages: usize,
    notes: usize,
    branches: usize,
    snapshots: usize,
}

#[derive(Debug, Clone)]
struct NotesStore {
    conversations_dir: PathBuf,
    messages_dir: PathBuf,
    notes_dir: PathBuf,
    branches_dir: PathBuf,
    snapshots_dir: PathBuf,
    exports_dir: PathBuf,
    index_path: PathBuf,
}

impl NotesStore {
    fn new(workspace: PathBuf) -> Result<Self> {
        let root = workspace.join(STORE_DIR);
        let conversations_dir = root.join("conversations");
        let messages_dir = root.join("messages");
        let notes_dir = root.join("notes");
        let branches_dir = root.join("branches");
        let snapshots_dir = root.join("snapshots");
        let exports_dir = root.join("exports");
        let index_path = root.join("index.json");

        std::fs::create_dir_all(&conversations_dir).with_context(|| {
            format!(
                "failed to create conversations dir at {}",
                conversations_dir.display()
            )
        })?;
        std::fs::create_dir_all(&messages_dir).with_context(|| {
            format!(
                "failed to create messages dir at {}",
                messages_dir.display()
            )
        })?;
        std::fs::create_dir_all(&notes_dir)
            .with_context(|| format!("failed to create notes dir at {}", notes_dir.display()))?;
        std::fs::create_dir_all(&branches_dir).with_context(|| {
            format!(
                "failed to create branches dir at {}",
                branches_dir.display()
            )
        })?;
        std::fs::create_dir_all(&snapshots_dir).with_context(|| {
            format!(
                "failed to create snapshots dir at {}",
                snapshots_dir.display()
            )
        })?;
        std::fs::create_dir_all(&exports_dir).with_context(|| {
            format!("failed to create exports dir at {}", exports_dir.display())
        })?;

        Ok(Self {
            conversations_dir,
            messages_dir,
            notes_dir,
            branches_dir,
            snapshots_dir,
            exports_dir,
            index_path,
        })
    }

    fn conversation_path(&self, id: &str) -> PathBuf {
        self.conversations_dir.join(format!("{id}.json"))
    }

    fn message_path(&self, id: &str) -> PathBuf {
        self.messages_dir.join(format!("{id}.json"))
    }

    fn note_path(&self, id: &str) -> PathBuf {
        self.notes_dir.join(format!("{id}.json"))
    }

    fn branch_path(&self, id: &str) -> PathBuf {
        self.branches_dir.join(format!("{id}.json"))
    }

    fn snapshot_path(&self, id: &str) -> PathBuf {
        self.snapshots_dir.join(format!("{id}.json"))
    }

    fn write_json<T: Serialize>(&self, path: &Path, value: &T) -> Result<()> {
        let json = serde_json::to_vec_pretty(value)
            .with_context(|| format!("failed to serialize json for {}", path.display()))?;
        let tmp = path.with_extension("json.tmp");
        std::fs::write(&tmp, json)
            .with_context(|| format!("failed to write temp json {}", tmp.display()))?;
        std::fs::rename(&tmp, path).with_context(|| {
            format!(
                "failed to replace {} with {}",
                path.display(),
                tmp.display()
            )
        })?;
        Ok(())
    }

    fn write_markdown(&self, path: &Path, content: &str) -> Result<()> {
        let tmp = path.with_extension("md.tmp");
        std::fs::write(&tmp, content)
            .with_context(|| format!("failed to write temp markdown {}", tmp.display()))?;
        std::fs::rename(&tmp, path).with_context(|| {
            format!(
                "failed to replace {} with {}",
                path.display(),
                tmp.display()
            )
        })?;
        Ok(())
    }

    fn read_json<T: for<'de> Deserialize<'de>>(&self, path: &Path) -> Result<T> {
        let raw = std::fs::read(path)
            .with_context(|| format!("failed to read json file {}", path.display()))?;
        serde_json::from_slice(&raw)
            .with_context(|| format!("failed to parse json file {}", path.display()))
    }

    fn list_json<T: for<'de> Deserialize<'de>>(&self, dir: &Path) -> Result<Vec<T>> {
        let mut entries = std::fs::read_dir(dir)
            .with_context(|| format!("failed to read directory {}", dir.display()))?
            .filter_map(|entry| entry.ok())
            .map(|entry| entry.path())
            .filter(|path| path.extension().is_some_and(|ext| ext == "json"))
            .collect::<Vec<_>>();
        entries.sort();

        let mut out = Vec::with_capacity(entries.len());
        for path in entries {
            out.push(self.read_json::<T>(&path)?);
        }
        Ok(out)
    }

    fn save_conversation(&self, conversation: &ConversationRecord) -> Result<()> {
        self.write_json(&self.conversation_path(&conversation.id), conversation)
    }

    fn save_message(&self, message: &MessageRecord) -> Result<()> {
        self.write_json(&self.message_path(&message.id), message)
    }

    fn save_note(&self, note: &NoteRecord) -> Result<()> {
        self.write_json(&self.note_path(&note.id), note)
    }

    fn save_branch(&self, branch: &BranchRecord) -> Result<()> {
        self.write_json(&self.branch_path(&branch.id), branch)
    }

    fn save_snapshot(&self, snapshot: &SnapshotRecord) -> Result<()> {
        self.write_json(&self.snapshot_path(&snapshot.id), snapshot)
    }

    fn load_conversation(&self, id: &str) -> Result<ConversationRecord> {
        let path = self.conversation_path(id);
        if !path.exists() {
            bail!("conversation not found: {id}");
        }
        self.read_json(&path)
    }

    fn load_note(&self, id: &str) -> Result<NoteRecord> {
        let path = self.note_path(id);
        if !path.exists() {
            bail!("note not found: {id}");
        }
        self.read_json(&path)
    }

    fn load_snapshot(&self, id: &str) -> Result<SnapshotRecord> {
        let path = self.snapshot_path(id);
        if !path.exists() {
            bail!("snapshot not found: {id}");
        }
        self.read_json(&path)
    }

    fn list_conversations(&self) -> Result<Vec<ConversationRecord>> {
        let mut rows = self.list_json::<ConversationRecord>(&self.conversations_dir)?;
        rows.sort_by_key(|row| std::cmp::Reverse(row.updated_at));
        Ok(rows)
    }

    fn list_messages(&self) -> Result<Vec<MessageRecord>> {
        let mut rows = self.list_json::<MessageRecord>(&self.messages_dir)?;
        rows.sort_by_key(|row| std::cmp::Reverse(row.created_at));
        Ok(rows)
    }

    fn list_notes(&self) -> Result<Vec<NoteRecord>> {
        let mut rows = self.list_json::<NoteRecord>(&self.notes_dir)?;
        rows.sort_by_key(|row| std::cmp::Reverse(row.updated_at));
        Ok(rows)
    }

    fn list_branches(&self) -> Result<Vec<BranchRecord>> {
        let mut rows = self.list_json::<BranchRecord>(&self.branches_dir)?;
        rows.sort_by_key(|row| row.created_at);
        Ok(rows)
    }

    fn list_snapshots(&self) -> Result<Vec<SnapshotRecord>> {
        let mut rows = self.list_json::<SnapshotRecord>(&self.snapshots_dir)?;
        rows.sort_by_key(|row| std::cmp::Reverse(row.created_at));
        Ok(rows)
    }

    fn find_message(
        &self,
        message_id: &str,
        conversation_id: Option<&str>,
    ) -> Result<(ConversationRecord, MessageRecord)> {
        let messages = self.list_messages()?;
        if let Some(conversation_id) = conversation_id {
            let message = messages
                .iter()
                .find(|row| row.id == message_id && row.conversation_id == conversation_id)
                .cloned()
                .with_context(|| {
                    format!("message {message_id} not found in conversation {conversation_id}")
                })?;
            return Ok((self.load_conversation(conversation_id)?, message));
        }

        let message = messages
            .into_iter()
            .find(|row| row.id == message_id)
            .with_context(|| format!("message not found: {message_id}"))?;
        let conversation = self.load_conversation(&message.conversation_id)?;
        Ok((conversation, message))
    }

    fn ensure_default_main_conversation(&self) -> Result<ConversationRecord> {
        let mut conversations = self.list_conversations()?;
        if let Some(found) = conversations
            .iter()
            .find(|conversation| conversation.title == "main")
            .cloned()
        {
            return Ok(found);
        }

        let now = now_ts();
        let conversation = ConversationRecord {
            id: new_id("c"),
            title: "main".to_string(),
            created_at: now,
            updated_at: now,
            root_message_id: None,
        };
        self.save_conversation(&conversation)?;
        conversations.push(conversation.clone());
        Ok(conversation)
    }

    fn rebuild_index(&self) -> Result<IndexSummary> {
        let summary = IndexSummary {
            version: VERSION,
            generated_at: now_ts(),
            conversations: self.list_conversations()?.len(),
            messages: self.list_messages()?.len(),
            notes: self.list_notes()?.len(),
            branches: self.list_branches()?.len(),
            snapshots: self.list_snapshots()?.len(),
        };
        self.write_json(&self.index_path, &summary)?;
        Ok(summary)
    }
}

pub fn run_conversation(cli: ConversationCli) -> Result<()> {
    match cli.subcommand {
        ConversationSubcommand::Create(args) => {
            let workspace = resolve_workspace(&args.workspace)?;
            let title = args.title.trim();
            if title.is_empty() {
                bail!("conversation title cannot be empty");
            }

            let store = NotesStore::new(workspace)?;
            let now = now_ts();
            let conversation = ConversationRecord {
                id: new_id("c"),
                title: title.to_string(),
                created_at: now,
                updated_at: now,
                root_message_id: None,
            };
            store.save_conversation(&conversation)?;
            let _ = store.rebuild_index()?;
            println!(
                "created conversation {} ({})",
                conversation.id, conversation.title
            );
        }
        ConversationSubcommand::List(args) => {
            let workspace = resolve_workspace(&args.workspace)?;
            let store = NotesStore::new(workspace)?;
            let conversations = store.list_conversations()?;
            if args.json {
                println!("{}", serde_json::to_string_pretty(&conversations)?);
            } else {
                for conversation in conversations {
                    let message_count = store
                        .list_messages()?
                        .iter()
                        .filter(|message| message.conversation_id == conversation.id)
                        .count();
                    println!(
                        "{}\t{}\t{}\tmessages={message_count}",
                        conversation.id, conversation.updated_at, conversation.title
                    );
                }
            }
        }
        ConversationSubcommand::Show(args) => {
            let workspace = resolve_workspace(&args.workspace)?;
            let store = NotesStore::new(workspace)?;
            let conversation = store.load_conversation(&args.id)?;
            let mut messages = store
                .list_messages()?
                .into_iter()
                .filter(|message| message.conversation_id == conversation.id)
                .collect::<Vec<_>>();
            messages.sort_by_key(|message| message.created_at);
            if args.json {
                let payload = serde_json::json!({
                    "conversation": conversation,
                    "messages": messages,
                });
                println!("{}", serde_json::to_string_pretty(&payload)?);
            } else {
                println!("{} {}", conversation.id, conversation.title);
                for message in messages {
                    println!(
                        "- [{}] {} {}: {}",
                        message.created_at, message.role, message.id, message.content
                    );
                }
            }
        }
    }

    Ok(())
}

pub fn run_message(cli: MessageCli) -> Result<()> {
    match cli.subcommand {
        MessageSubcommand::Add(args) => {
            let workspace = resolve_workspace(&args.workspace)?;
            let store = NotesStore::new(workspace)?;
            let mut conversation = store.load_conversation(&args.conversation)?;
            let role = args.role.trim().to_lowercase();
            if !["user", "assistant", "system", "tool"].contains(&role.as_str()) {
                bail!("role must be one of user, assistant, system, tool");
            }
            let content = args.content.trim();
            if content.is_empty() {
                bail!("message content cannot be empty");
            }

            if let Some(parent_id) = &args.parent {
                let has_parent = store.list_messages()?.iter().any(|message| {
                    message.id == *parent_id && message.conversation_id == conversation.id
                });
                if !has_parent {
                    bail!("parent message not found: {parent_id}");
                }
            }

            let message = MessageRecord {
                id: new_id("m"),
                conversation_id: conversation.id.clone(),
                parent_id: args.parent,
                role,
                content: content.to_string(),
                created_at: now_ts(),
            };

            if conversation.root_message_id.is_none() {
                conversation.root_message_id = Some(message.id.clone());
            }
            conversation.updated_at = message.created_at;

            store.save_message(&message)?;
            store.save_conversation(&conversation)?;
            let _ = store.rebuild_index()?;

            if args.json {
                println!("{}", serde_json::to_string_pretty(&message)?);
            } else {
                println!(
                    "added message {} to {}",
                    message.id, message.conversation_id
                );
            }
        }
    }

    Ok(())
}

pub fn run_note(cli: NoteCli) -> Result<()> {
    match cli.subcommand {
        NoteSubcommand::Add(args) => {
            let workspace = resolve_workspace(&args.workspace)?;
            let store = NotesStore::new(workspace.clone())?;
            let title = args.title.trim();
            let body = args.body.trim();
            if title.is_empty() {
                bail!("note title cannot be empty");
            }
            if body.is_empty() {
                bail!("note body cannot be empty");
            }

            let status = normalize_status(args.status)?;
            let priority = normalize_priority(args.priority)?;
            let tags = clean_tags(args.tags);

            let conversation_id =
                resolve_note_conversation_id(&store, args.conversation, args.message.clone())?;
            let message_id = args.message;
            let now = now_ts();
            let note = NoteRecord {
                id: new_id("n"),
                conversation_id,
                message_id,
                title: title.to_string(),
                body: body.to_string(),
                tags,
                status,
                priority,
                created_at: now,
                updated_at: now,
                repo_ctx: Some(capture_repo_context(&workspace, args.related_files)),
            };
            store.save_note(&note)?;
            let _ = store.rebuild_index()?;

            if args.json {
                println!("{}", serde_json::to_string_pretty(&note)?);
            } else {
                println!("created note {} in {}", note.id, note.conversation_id);
            }
        }
        NoteSubcommand::Annotate(args) => {
            let workspace = resolve_workspace(&args.workspace)?;
            let store = NotesStore::new(workspace.clone())?;
            let body = args.body.trim();
            if body.is_empty() {
                bail!("annotation body cannot be empty");
            }
            let (conversation, message) =
                store.find_message(&args.message, args.conversation.as_deref())?;
            let mut tags = clean_tags(args.tags);
            if !tags.iter().any(|tag| tag == "annotation") {
                tags.push("annotation".to_string());
            }
            let now = now_ts();
            let note = NoteRecord {
                id: new_id("n"),
                conversation_id: conversation.id,
                message_id: Some(message.id.clone()),
                title: format!("Annotation: {}", message.id),
                body: body.to_string(),
                tags,
                status: "open".to_string(),
                priority: "p2".to_string(),
                created_at: now,
                updated_at: now,
                repo_ctx: Some(capture_repo_context(&workspace, Vec::new())),
            };
            store.save_note(&note)?;
            let _ = store.rebuild_index()?;

            if args.json {
                println!("{}", serde_json::to_string_pretty(&note)?);
            } else {
                println!("annotated message {} with note {}", message.id, note.id);
            }
        }
        NoteSubcommand::List(args) => {
            let workspace = resolve_workspace(&args.workspace)?;
            let store = NotesStore::new(workspace.clone())?;
            let status = match args.status {
                Some(status) => Some(normalize_status(status)?),
                None => None,
            };
            let tag = args.tag;
            let conversation = args.conversation;
            let repo = args.repo;
            let current_repo = if repo.as_deref() == Some("current") {
                Some(capture_repo_context(&workspace, Vec::new()))
            } else {
                None
            };

            let mut notes = store.list_notes()?;
            notes.retain(|note| {
                if status
                    .as_ref()
                    .is_some_and(|expected| expected != &note.status)
                {
                    return false;
                }
                if tag
                    .as_ref()
                    .is_some_and(|expected| !note.tags.iter().any(|row| row == expected))
                {
                    return false;
                }
                if conversation
                    .as_ref()
                    .is_some_and(|expected| expected != &note.conversation_id)
                {
                    return false;
                }
                if let Some(repo) = repo.as_ref() {
                    if repo == "current" {
                        let Some(current_repo) = current_repo.as_ref() else {
                            return false;
                        };
                        return note
                            .repo_ctx
                            .as_ref()
                            .is_some_and(|ctx| ctx.repo_path == current_repo.repo_path);
                    }
                    return note
                        .repo_ctx
                        .as_ref()
                        .is_some_and(|ctx| &ctx.repo_path == repo);
                }
                true
            });

            if args.json {
                println!("{}", serde_json::to_string_pretty(&notes)?);
            } else {
                for note in notes {
                    let tags = if note.tags.is_empty() {
                        "-".to_string()
                    } else {
                        note.tags.join(",")
                    };
                    println!(
                        "{}\t{}\t{}\t{}\t{}\t{}\t{}",
                        note.id,
                        note.updated_at,
                        note.status,
                        note.priority,
                        note.conversation_id,
                        tags,
                        note.title
                    );
                }
            }
        }
        NoteSubcommand::Show(args) => {
            let workspace = resolve_workspace(&args.workspace)?;
            let store = NotesStore::new(workspace)?;
            let note = store.load_note(&args.id)?;
            if args.json {
                println!("{}", serde_json::to_string_pretty(&note)?);
            } else {
                println!("{} {}", note.id, note.title);
                println!("conversation: {}", note.conversation_id);
                println!("status: {} priority: {}", note.status, note.priority);
                println!(
                    "tags: {}",
                    if note.tags.is_empty() {
                        "(none)".to_string()
                    } else {
                        note.tags.join(", ")
                    }
                );
                println!("{}", note.body);
            }
        }
        NoteSubcommand::Update(args) => {
            let workspace = resolve_workspace(&args.workspace)?;
            let store = NotesStore::new(workspace)?;
            let mut note = store.load_note(&args.id)?;

            if let Some(title) = args.title {
                let trimmed = title.trim();
                if trimmed.is_empty() {
                    bail!("note title cannot be empty");
                }
                note.title = trimmed.to_string();
            }
            if let Some(body) = args.body {
                let trimmed = body.trim();
                if trimmed.is_empty() {
                    bail!("note body cannot be empty");
                }
                note.body = trimmed.to_string();
            }
            if args.clear_tags {
                note.tags.clear();
            } else if !args.tags.is_empty() {
                note.tags = clean_tags(args.tags);
            }
            if let Some(status) = args.status {
                note.status = normalize_status(status)?;
            }
            if let Some(priority) = args.priority {
                note.priority = normalize_priority(priority)?;
            }
            note.updated_at = now_ts();

            store.save_note(&note)?;
            let _ = store.rebuild_index()?;
            if args.json {
                println!("{}", serde_json::to_string_pretty(&note)?);
            } else {
                println!("updated note {}", note.id);
            }
        }
        NoteSubcommand::Archive(args) => {
            let workspace = resolve_workspace(&args.workspace)?;
            let store = NotesStore::new(workspace)?;
            let mut note = store.load_note(&args.id)?;
            note.status = "archived".to_string();
            note.updated_at = now_ts();
            store.save_note(&note)?;
            let _ = store.rebuild_index()?;
            if args.json {
                println!("{}", serde_json::to_string_pretty(&note)?);
            } else {
                println!("archived note {}", note.id);
            }
        }
    }

    Ok(())
}

pub fn run_branch(cli: BranchCli) -> Result<()> {
    match cli.subcommand {
        BranchSubcommand::Fork(args) => {
            let workspace = resolve_workspace(&args.workspace)?;
            let store = NotesStore::new(workspace)?;
            let (source_conversation, source_message) =
                store.find_message(&args.message, args.conversation.as_deref())?;

            let now = now_ts();
            let mut new_conversation = ConversationRecord {
                id: new_id("c"),
                title: args
                    .title
                    .as_deref()
                    .map(str::trim)
                    .filter(|title| !title.is_empty())
                    .map(ToOwned::to_owned)
                    .unwrap_or_else(|| format!("branch-{}", source_message.id)),
                created_at: now,
                updated_at: now,
                root_message_id: None,
            };

            let root_message = MessageRecord {
                id: new_id("m"),
                conversation_id: new_conversation.id.clone(),
                parent_id: None,
                role: source_message.role,
                content: source_message.content,
                created_at: now,
            };
            let system_message = MessageRecord {
                id: new_id("m"),
                conversation_id: new_conversation.id.clone(),
                parent_id: Some(root_message.id.clone()),
                role: "system".to_string(),
                content: format!(
                    "Forked from {}:{}",
                    source_conversation.id, source_message.id
                ),
                created_at: now,
            };
            new_conversation.root_message_id = Some(root_message.id.clone());
            new_conversation.updated_at = system_message.created_at;

            let branch = BranchRecord {
                id: new_id("b"),
                source_conversation_id: source_conversation.id,
                source_message_id: source_message.id,
                new_conversation_id: new_conversation.id.clone(),
                created_at: now,
            };

            store.save_conversation(&new_conversation)?;
            store.save_message(&root_message)?;
            store.save_message(&system_message)?;
            store.save_branch(&branch)?;
            let _ = store.rebuild_index()?;

            if args.json {
                let payload = serde_json::json!({
                    "branch": branch,
                    "conversation": new_conversation,
                });
                println!("{}", serde_json::to_string_pretty(&payload)?);
            } else {
                println!(
                    "forked branch {}: {}:{} -> {}",
                    branch.id,
                    branch.source_conversation_id,
                    branch.source_message_id,
                    branch.new_conversation_id
                );
            }
        }
        BranchSubcommand::Tree(args) => {
            let workspace = resolve_workspace(&args.workspace)?;
            let store = NotesStore::new(workspace)?;
            let conversations = store.list_conversations()?;
            let conversation_map = conversations
                .into_iter()
                .map(|conversation| (conversation.id.clone(), conversation))
                .collect::<BTreeMap<_, _>>();

            if !conversation_map.contains_key(&args.conversation) {
                bail!("conversation not found: {}", args.conversation);
            }

            let mut children = BTreeMap::<String, Vec<String>>::new();
            for branch in store.list_branches()? {
                children
                    .entry(branch.source_conversation_id)
                    .or_default()
                    .push(branch.new_conversation_id);
            }

            for items in children.values_mut() {
                items.sort();
            }

            let mut lines = Vec::new();
            render_branch_tree(
                &args.conversation,
                0,
                &mut BTreeSet::new(),
                &conversation_map,
                &children,
                &mut lines,
            );
            println!("{}", lines.join("\n"));
        }
    }

    Ok(())
}

pub fn run_snapshot(cli: SnapshotCli) -> Result<()> {
    match cli.subcommand {
        SnapshotSubcommand::Create(args) => {
            let workspace = resolve_workspace(&args.workspace)?;
            let store = NotesStore::new(workspace.clone())?;
            let conversation = store.load_conversation(&args.conversation)?;
            let summary = if let Some(summary) = args.summary {
                let summary = summary.trim().to_string();
                if summary.is_empty() {
                    bail!("snapshot summary cannot be empty");
                }
                summary
            } else if args.from_latest {
                build_latest_summary(&store, &conversation.id)?
            } else {
                bail!("snapshot summary cannot be empty; use --summary or --from-latest");
            };

            let todo = clean_items(args.todo);
            let risks = clean_items(args.risk);
            let snapshot = SnapshotRecord {
                id: new_id("s"),
                conversation_id: conversation.id,
                summary,
                todo,
                risks,
                repo_ctx: Some(capture_repo_context(&workspace, Vec::new())),
                created_at: now_ts(),
            };

            store.save_snapshot(&snapshot)?;
            let _ = store.rebuild_index()?;
            if args.json {
                println!("{}", serde_json::to_string_pretty(&snapshot)?);
            } else {
                println!(
                    "created snapshot {} for {}",
                    snapshot.id, snapshot.conversation_id
                );
            }
        }
        SnapshotSubcommand::List(args) => {
            let workspace = resolve_workspace(&args.workspace)?;
            let store = NotesStore::new(workspace)?;
            let mut snapshots = store.list_snapshots()?;
            if let Some(conversation_id) = args.conversation {
                snapshots.retain(|snapshot| snapshot.conversation_id == conversation_id);
            }

            if args.json {
                println!("{}", serde_json::to_string_pretty(&snapshots)?);
            } else {
                for snapshot in snapshots {
                    println!(
                        "{}\t{}\t{}\t{}",
                        snapshot.id,
                        snapshot.created_at,
                        snapshot.conversation_id,
                        trim_for_table(&snapshot.summary, 80)
                    );
                }
            }
        }
        SnapshotSubcommand::Show(args) => {
            let workspace = resolve_workspace(&args.workspace)?;
            let store = NotesStore::new(workspace)?;
            let snapshot = store.load_snapshot(&args.id)?;
            if args.json {
                println!("{}", serde_json::to_string_pretty(&snapshot)?);
            } else {
                println!("{}", render_resume_text(&store, &snapshot)?);
            }
        }
        SnapshotSubcommand::Resume(args) => {
            let workspace = resolve_workspace(&args.workspace)?;
            let store = NotesStore::new(workspace)?;
            let snapshot = store.load_snapshot(&args.snapshot)?;
            println!("{}", render_resume_text(&store, &snapshot)?);
        }
    }

    Ok(())
}

pub fn run_search(args: SearchArgs) -> Result<()> {
    let workspace = resolve_workspace(&args.workspace)?;
    let store = NotesStore::new(workspace.clone())?;
    let query = args.query.trim().to_lowercase();
    if query.is_empty() {
        bail!("query cannot be empty");
    }

    let status = match args.status {
        Some(status) => Some(normalize_status(status)?),
        None => None,
    };

    let notes = store.list_notes()?;
    let snapshots = store.list_snapshots()?;
    let messages = store.list_messages()?;

    let current_repo = if args.repo.as_deref() == Some("current") {
        Some(capture_repo_context(&workspace, Vec::new()))
    } else {
        None
    };

    let mut rows = Vec::new();

    for note in notes {
        if args
            .conversation
            .as_ref()
            .is_some_and(|conversation| conversation != &note.conversation_id)
        {
            continue;
        }

        if status.as_ref().is_some_and(|status| status != &note.status) {
            continue;
        }

        if args
            .tag
            .as_ref()
            .is_some_and(|tag| !note.tags.iter().any(|value| value == tag))
        {
            continue;
        }

        if let Some(repo) = &args.repo {
            if repo == "current" {
                let Some(current_repo) = current_repo.as_ref() else {
                    continue;
                };
                if !note
                    .repo_ctx
                    .as_ref()
                    .is_some_and(|ctx| ctx.repo_path == current_repo.repo_path)
                {
                    continue;
                }
            } else if !note
                .repo_ctx
                .as_ref()
                .is_some_and(|ctx| &ctx.repo_path == repo)
            {
                continue;
            }
        }

        let hay = format!("{} {} {}", note.title, note.body, note.tags.join(" ")).to_lowercase();
        if !hay.contains(&query) {
            continue;
        }

        rows.push(SearchResultRow {
            kind: "note".to_string(),
            id: note.id,
            conversation_id: note.conversation_id,
            title: note.title,
            snippet: trim_for_table(&note.body, 120),
            updated_at: note.updated_at,
        });
    }

    for snapshot in snapshots {
        if args
            .conversation
            .as_ref()
            .is_some_and(|conversation| conversation != &snapshot.conversation_id)
        {
            continue;
        }

        if let Some(repo) = &args.repo {
            if repo == "current" {
                let Some(current_repo) = current_repo.as_ref() else {
                    continue;
                };
                if !snapshot
                    .repo_ctx
                    .as_ref()
                    .is_some_and(|ctx| ctx.repo_path == current_repo.repo_path)
                {
                    continue;
                }
            } else if !snapshot
                .repo_ctx
                .as_ref()
                .is_some_and(|ctx| &ctx.repo_path == repo)
            {
                continue;
            }
        }

        let hay = format!(
            "{} {} {}",
            snapshot.summary,
            snapshot.todo.join(" "),
            snapshot.risks.join(" ")
        )
        .to_lowercase();
        if !hay.contains(&query) {
            continue;
        }

        rows.push(SearchResultRow {
            kind: "snapshot".to_string(),
            id: snapshot.id,
            conversation_id: snapshot.conversation_id,
            title: trim_for_table(&snapshot.summary, 80),
            snippet: trim_for_table(&snapshot.summary, 120),
            updated_at: snapshot.created_at,
        });
    }

    for message in messages {
        if args
            .conversation
            .as_ref()
            .is_some_and(|conversation| conversation != &message.conversation_id)
        {
            continue;
        }

        if !message.content.to_lowercase().contains(&query) {
            continue;
        }

        rows.push(SearchResultRow {
            kind: "message".to_string(),
            id: message.id,
            conversation_id: message.conversation_id,
            title: format!("{} message", message.role),
            snippet: trim_for_table(&message.content, 120),
            updated_at: message.created_at,
        });
    }

    rows.sort_by_key(|row| std::cmp::Reverse(row.updated_at));

    if args.json {
        println!("{}", serde_json::to_string_pretty(&rows)?);
    } else {
        for row in rows {
            println!(
                "{}\t{}\t{}\t{}\t{}",
                row.kind, row.id, row.conversation_id, row.title, row.snippet
            );
        }
    }

    Ok(())
}

pub fn run_export(cli: ExportCli) -> Result<()> {
    match cli.subcommand {
        ExportSubcommand::Conversation(args) => {
            if args.format != "md" {
                bail!("only md format is supported");
            }

            let workspace = resolve_workspace(&args.workspace)?;
            let store = NotesStore::new(workspace)?;
            let root_conversation = store.load_conversation(&args.id)?;
            let mut conversation_ids = BTreeSet::new();
            conversation_ids.insert(root_conversation.id.clone());

            if args.with_branches {
                let branches = store.list_branches()?;
                let mut queue = vec![root_conversation.id.clone()];
                while let Some(conversation_id) = queue.pop() {
                    for branch in branches
                        .iter()
                        .filter(|branch| branch.source_conversation_id == conversation_id)
                    {
                        if conversation_ids.insert(branch.new_conversation_id.clone()) {
                            queue.push(branch.new_conversation_id.clone());
                        }
                    }
                }
            }

            let all_conversations = store
                .list_conversations()?
                .into_iter()
                .map(|conversation| (conversation.id.clone(), conversation))
                .collect::<BTreeMap<_, _>>();
            let mut messages = store
                .list_messages()?
                .into_iter()
                .filter(|message| conversation_ids.contains(&message.conversation_id))
                .collect::<Vec<_>>();
            let notes = store
                .list_notes()?
                .into_iter()
                .filter(|note| conversation_ids.contains(&note.conversation_id))
                .collect::<Vec<_>>();
            let snapshots = store
                .list_snapshots()?
                .into_iter()
                .filter(|snapshot| conversation_ids.contains(&snapshot.conversation_id))
                .collect::<Vec<_>>();

            messages.sort_by_key(|message| message.created_at);

            let mut markdown = String::new();
            markdown.push_str(&format!(
                "# Conversation Export: {}\n\n",
                root_conversation.id
            ));
            markdown.push_str(&format!("Title: {}\n", root_conversation.title));
            markdown.push_str(&format!("Exported At: {}\n", now_ts()));
            markdown.push_str(&format!(
                "Include Branches: {}\n\n",
                if args.with_branches { "yes" } else { "no" }
            ));

            markdown.push_str("## Conversations\n\n");
            for conversation_id in &conversation_ids {
                if let Some(conversation) = all_conversations.get(conversation_id) {
                    markdown.push_str(&format!(
                        "### {} - {}\n\n",
                        conversation.id, conversation.title
                    ));
                    for message in messages
                        .iter()
                        .filter(|message| message.conversation_id == conversation.id)
                    {
                        markdown.push_str(&format!(
                            "- [{}] {}: {}\n",
                            message.created_at, message.role, message.content
                        ));
                    }
                    markdown.push('\n');
                }
            }

            markdown.push_str("## Notes\n\n");
            for note in notes {
                markdown.push_str(&format!("### {} - {}\n", note.id, note.title));
                markdown.push_str(&format!("- conversation: {}\n", note.conversation_id));
                markdown.push_str(&format!("- status: {}\n", note.status));
                markdown.push_str(&format!("- priority: {}\n", note.priority));
                markdown.push_str(&format!(
                    "- tags: {}\n",
                    if note.tags.is_empty() {
                        "(none)".to_string()
                    } else {
                        note.tags.join(", ")
                    }
                ));
                markdown.push_str(&format!("{}\n\n", note.body));
            }

            markdown.push_str("## Snapshots\n\n");
            for snapshot in snapshots {
                markdown.push_str(&format!("### {}\n", snapshot.id));
                markdown.push_str(&format!("- conversation: {}\n", snapshot.conversation_id));
                markdown.push_str(&format!("- created_at: {}\n", snapshot.created_at));
                markdown.push_str(&format!("- summary: {}\n", snapshot.summary));
                markdown.push_str(&format!(
                    "- todo: {}\n",
                    if snapshot.todo.is_empty() {
                        "(none)".to_string()
                    } else {
                        snapshot.todo.join(", ")
                    }
                ));
                markdown.push_str(&format!(
                    "- risks: {}\n\n",
                    if snapshot.risks.is_empty() {
                        "(none)".to_string()
                    } else {
                        snapshot.risks.join(", ")
                    }
                ));
            }

            let filename = format!("{}-{}.md", args.id, now_ts());
            let path = store.exports_dir.join(filename);
            store.write_markdown(&path, &markdown)?;
            let _ = store.rebuild_index()?;

            if args.json {
                let payload = serde_json::json!({ "path": path.display().to_string() });
                println!("{}", serde_json::to_string_pretty(&payload)?);
            } else {
                println!("exported: {}", path.display());
            }
        }
    }

    Ok(())
}

pub fn run_index(cli: IndexCli) -> Result<()> {
    match cli.subcommand {
        IndexSubcommand::Rebuild(args) => {
            let workspace = resolve_workspace(&args.workspace)?;
            let store = NotesStore::new(workspace)?;
            let summary = store.rebuild_index()?;
            if args.json {
                println!("{}", serde_json::to_string_pretty(&summary)?);
            } else {
                println!(
                    "rebuilt index: conversations={} messages={} notes={} branches={} snapshots={}",
                    summary.conversations,
                    summary.messages,
                    summary.notes,
                    summary.branches,
                    summary.snapshots
                );
            }
        }
    }

    Ok(())
}

fn resolve_workspace(workspace: &Path) -> Result<PathBuf> {
    if workspace.exists() {
        return workspace
            .canonicalize()
            .with_context(|| format!("failed to resolve workspace {}", workspace.display()));
    }

    bail!("workspace does not exist: {}", workspace.display())
}

fn now_ts() -> i64 {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    i64::try_from(secs).unwrap_or(i64::MAX)
}

fn new_id(prefix: &str) -> String {
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    let counter = ID_COUNTER.fetch_add(1, Ordering::Relaxed);
    format!("{prefix}_{millis}_{counter}")
}

fn normalize_status(status: String) -> Result<String> {
    let status = status.trim().to_lowercase();
    if NOTE_STATUSES.iter().any(|candidate| *candidate == status) {
        return Ok(status);
    }

    bail!(
        "invalid status: {status}; allowed values: {}",
        NOTE_STATUSES.join(", ")
    )
}

fn normalize_priority(priority: String) -> Result<String> {
    let priority = priority.trim().to_lowercase();
    if NOTE_PRIORITIES
        .iter()
        .any(|candidate| *candidate == priority)
    {
        return Ok(priority);
    }

    bail!(
        "invalid priority: {priority}; allowed values: {}",
        NOTE_PRIORITIES.join(", ")
    )
}

fn clean_tags(tags: Vec<String>) -> Vec<String> {
    tags.into_iter()
        .map(|tag| tag.trim().to_lowercase())
        .filter(|tag| !tag.is_empty())
        .collect()
}

fn clean_items(items: Vec<String>) -> Vec<String> {
    items
        .into_iter()
        .map(|item| item.trim().to_string())
        .filter(|item| !item.is_empty())
        .collect()
}

fn trim_for_table(value: &str, max_len: usize) -> String {
    if value.len() <= max_len {
        return value.to_string();
    }

    let mut out = value
        .chars()
        .take(max_len.saturating_sub(1))
        .collect::<String>();
    out.push('…');
    out
}

fn capture_repo_context(workspace: &Path, related_files: Vec<String>) -> RepoContext {
    let repo_root = run_git(workspace, &["rev-parse", "--show-toplevel"]);
    let (repo_path, git_branch, git_commit) = if let Some(repo_root) = repo_root {
        (
            repo_root,
            run_git(workspace, &["branch", "--show-current"]),
            run_git(workspace, &["rev-parse", "HEAD"]),
        )
    } else {
        (workspace.display().to_string(), None, None)
    };

    RepoContext {
        repo_path,
        git_branch,
        git_commit,
        related_files,
    }
}

fn run_git(workspace: &Path, args: &[&str]) -> Option<String> {
    let output = Command::new("git")
        .args(args)
        .current_dir(workspace)
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let text = String::from_utf8(output.stdout).ok()?;
    let value = text.trim().to_string();
    if value.is_empty() { None } else { Some(value) }
}

fn resolve_note_conversation_id(
    store: &NotesStore,
    conversation_id: Option<String>,
    message_id: Option<String>,
) -> Result<String> {
    if let (Some(conversation_id), Some(message_id)) =
        (conversation_id.as_ref(), message_id.as_ref())
    {
        let _ = store.find_message(message_id, Some(conversation_id))?;
        return Ok(conversation_id.to_string());
    }

    if let Some(message_id) = message_id {
        let (conversation, _) = store.find_message(&message_id, None)?;
        return Ok(conversation.id);
    }

    if let Some(conversation_id) = conversation_id {
        let _ = store.load_conversation(&conversation_id)?;
        return Ok(conversation_id);
    }

    Ok(store.ensure_default_main_conversation()?.id)
}

fn build_latest_summary(store: &NotesStore, conversation_id: &str) -> Result<String> {
    let mut messages = store
        .list_messages()?
        .into_iter()
        .filter(|message| message.conversation_id == conversation_id)
        .collect::<Vec<_>>();
    messages.sort_by_key(|message| std::cmp::Reverse(message.created_at));

    let notes = store
        .list_notes()?
        .into_iter()
        .filter(|note| note.conversation_id == conversation_id)
        .take(3)
        .collect::<Vec<_>>();

    let mut chunks = Vec::new();
    if !messages.is_empty() {
        let message_block = messages
            .iter()
            .take(3)
            .map(|message| format!("{}:{}", message.role, trim_for_table(&message.content, 60)))
            .collect::<Vec<_>>()
            .join(" | ");
        chunks.push(format!("Latest messages: {message_block}"));
    }

    if !notes.is_empty() {
        let note_block = notes
            .iter()
            .map(|note| note.title.clone())
            .collect::<Vec<_>>()
            .join(" | ");
        chunks.push(format!("Latest notes: {note_block}"));
    }

    if chunks.is_empty() {
        chunks.push("No recent messages or notes. Snapshot created as a baseline.".to_string());
    }

    Ok(chunks.join("\n"))
}

fn render_resume_text(store: &NotesStore, snapshot: &SnapshotRecord) -> Result<String> {
    let conversation = store.load_conversation(&snapshot.conversation_id)?;

    let mut lines = vec![
        format!("Snapshot: {}", snapshot.id),
        format!("Conversation: {} ({})", conversation.id, conversation.title),
        format!("Created At: {}", snapshot.created_at),
        String::new(),
        "Summary:".to_string(),
        snapshot.summary.clone(),
        String::new(),
        "TODO:".to_string(),
    ];

    if snapshot.todo.is_empty() {
        lines.push("- (none)".to_string());
    } else {
        lines.extend(snapshot.todo.iter().map(|item| format!("- {item}")));
    }

    lines.push(String::new());
    lines.push("Risks:".to_string());
    if snapshot.risks.is_empty() {
        lines.push("- (none)".to_string());
    } else {
        lines.extend(snapshot.risks.iter().map(|item| format!("- {item}")));
    }

    lines.push(String::new());
    lines.push("Repo Context:".to_string());
    if let Some(repo_ctx) = &snapshot.repo_ctx {
        lines.push(format!("- repo_path: {}", repo_ctx.repo_path));
        lines.push(format!(
            "- git_branch: {}",
            repo_ctx.git_branch.as_deref().unwrap_or("(unknown)")
        ));
        lines.push(format!(
            "- git_commit: {}",
            repo_ctx.git_commit.as_deref().unwrap_or("(unknown)")
        ));
    } else {
        lines.push("- (missing)".to_string());
    }

    Ok(lines.join("\n"))
}

fn render_branch_tree(
    conversation_id: &str,
    depth: usize,
    seen: &mut BTreeSet<String>,
    conversation_map: &BTreeMap<String, ConversationRecord>,
    children: &BTreeMap<String, Vec<String>>,
    out: &mut Vec<String>,
) {
    let prefix = "  ".repeat(depth);
    let label = conversation_map
        .get(conversation_id)
        .map(|conversation| conversation.title.clone())
        .unwrap_or_else(|| conversation_id.to_string());
    out.push(format!("{prefix}- {conversation_id} ({label})"));

    if !seen.insert(conversation_id.to_string()) {
        out.push(format!("{prefix}  - [cycle detected]"));
        return;
    }

    if let Some(next_conversations) = children.get(conversation_id) {
        for next in next_conversations {
            render_branch_tree(next, depth + 1, seen, conversation_map, children, out);
        }
    }

    seen.remove(conversation_id);
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use tempfile::tempdir;

    #[test]
    fn end_to_end_flow_works() {
        let tmp = tempdir().expect("tempdir");
        let workspace = tmp.path().to_path_buf();

        let store = NotesStore::new(workspace.clone()).expect("store");

        let conversation = ConversationRecord {
            id: "c_main".to_string(),
            title: "main".to_string(),
            created_at: 1,
            updated_at: 1,
            root_message_id: None,
        };
        store
            .save_conversation(&conversation)
            .expect("save conversation");

        let message = MessageRecord {
            id: "m1".to_string(),
            conversation_id: conversation.id.clone(),
            parent_id: None,
            role: "user".to_string(),
            content: "Need rollback strategy".to_string(),
            created_at: 2,
        };
        store.save_message(&message).expect("save message");

        let note = NoteRecord {
            id: "n1".to_string(),
            conversation_id: conversation.id.clone(),
            message_id: Some(message.id.clone()),
            title: "plan".to_string(),
            body: "use feature flag".to_string(),
            tags: vec!["risk".to_string()],
            status: "open".to_string(),
            priority: "p1".to_string(),
            created_at: 3,
            updated_at: 3,
            repo_ctx: None,
        };
        store.save_note(&note).expect("save note");

        let snapshot = SnapshotRecord {
            id: "s1".to_string(),
            conversation_id: conversation.id.clone(),
            summary: "prepared".to_string(),
            todo: vec!["staging".to_string()],
            risks: vec!["missing tests".to_string()],
            repo_ctx: None,
            created_at: 4,
        };
        store.save_snapshot(&snapshot).expect("save snapshot");

        let summary = store.rebuild_index().expect("rebuild index");
        assert_eq!(summary.conversations, 1);
        assert_eq!(summary.messages, 1);
        assert_eq!(summary.notes, 1);
        assert_eq!(summary.snapshots, 1);

        let resume = render_resume_text(&store, &snapshot).expect("resume text");
        assert!(resume.contains("Snapshot: s1"));
        assert!(!resume.contains("Need rollback strategy"));
    }

    #[test]
    fn validate_status_and_priority() {
        assert_eq!(
            normalize_status("open".to_string()).expect("status"),
            "open"
        );
        assert_eq!(
            normalize_priority("p2".to_string()).expect("priority"),
            "p2"
        );
        assert!(normalize_status("nope".to_string()).is_err());
        assert!(normalize_priority("hi".to_string()).is_err());
    }

    #[test]
    fn trimming_keeps_short_strings() {
        assert_eq!(trim_for_table("abc", 10), "abc");
        let trimmed = trim_for_table("123456", 4);
        assert!(trimmed.ends_with('…'));
    }
}

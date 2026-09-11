// SPDX-License-Identifier: AGPL-3.0-or-later
//! Protocol identity newtypes. Owner chunk: **C-07**. Intentionally empty.
//!
//! C-07 lands every identity in one PR (COLLISION-3): definition, role instance,
//! workflow instance, participant, component, endpoint, channel, message, send,
//! receive, delivery, attempt and effect, plus the registration, commit and
//! recovery identities. Each wraps the producer's exact identity bytes, none
//! derives `From<String>`, and display text is never identity. Later chunks
//! consume these types and never extend this file.

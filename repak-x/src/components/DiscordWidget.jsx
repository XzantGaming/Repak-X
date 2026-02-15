import React, { useState, useEffect } from 'react';
import { open as openUrl } from '@tauri-apps/plugin-shell';
import repakIcon from '../assets/app-icons/RepakIcon-x256.png';

const GUILD_ID = '1448416689640181862';
const INVITE_URL = 'https://discord.gg/placeholder';

const CommunityBadge = () => (
  <div style={{ position: 'relative', width: 16, height: 16, flexShrink: 0 }}>
    <svg aria-hidden="true" width="16" height="16" viewBox="0 0 16 16" style={{ position: 'absolute', top: 0, left: 0 }}>
      <path fill="#5865F2" fillRule="evenodd" clipRule="evenodd" d="M5.52995 0.431867C6.27995 0.181867 7.2 1.23199 8 1.23199C8.8 1.23199 9.75007 0.231867 10.4701 0.431867C11.19 0.631965 11.38 2.07173 12 2.52171C12.62 2.9717 14.0003 2.70184 14.4603 3.32184C14.9199 3.94191 14.2303 5.16178 14.4603 5.91169C14.6903 6.66159 15.9998 7.21166 16 8.00153C16 8.79146 14.72 9.38146 14.4798 10.0914C14.2398 10.8014 14.9198 12.0919 14.4798 12.6819C14.0397 13.2716 12.6401 13.0321 12.0202 13.482C11.4002 13.932 11.2298 15.3219 10.4798 15.5719C9.72987 15.8216 8.80967 14.7717 8.00973 14.7717C7.2098 14.7719 6.25964 15.7718 5.53971 15.5719C4.81995 15.3716 4.61982 13.9321 4 13.482C3.38 13.032 1.99971 13.3019 1.53971 12.6819C1.07975 12.0619 1.76971 10.8414 1.53971 10.0914C1.30939 9.34159 0 8.79139 0 8.00153C0.000177681 7.21159 1.2802 6.62163 1.52018 5.91169C1.76012 5.20167 1.08021 3.91183 1.52018 3.32184C1.96018 2.73184 3.36999 3.0017 4 2.52171C4.62997 2.04173 4.78003 0.681954 5.52995 0.431867Z" />
    </svg>
    <svg aria-hidden="true" width="11" height="11" viewBox="0 0 24 24" fill="none" style={{ position: 'absolute', top: '50%', left: '50%', transform: 'translate(-50%, -50%)' }}>
      <path fill="#fff" d="m2.4 8.4 8.38-6.46a2 2 0 0 1 2.44 0l8.39 6.45a2 2 0 0 1-.79 3.54l-.32.07-.82 8.2a2 2 0 0 1-1.99 1.8H16a1 1 0 0 1-1-1v-5a3 3 0 1 0-6 0v5a1 1 0 0 1-1 1H6.31a2 2 0 0 1-1.99-1.8L3.5 12l-.32-.07a2 2 0 0 1-.79-3.54Z" />
    </svg>
  </div>
);

export default function DiscordWidget() {
  const [widgetData, setWidgetData] = useState(null);

  useEffect(() => {
    fetch(`https://discord.com/api/guilds/${GUILD_ID}/widget.json`)
      .then(res => {
        if (!res.ok) throw new Error('Widget not available');
        return res.json();
      })
      .then(data => setWidgetData(data))
      .catch(() => { });
  }, []);

  const onlineCount = widgetData?.presence_count ?? '—';
  const serverName = widgetData?.name ?? 'REPAK X';

  return (
    <div style={{
      borderRadius: 10,
      overflow: 'hidden',
      background: '#2b2d31',
      border: '1px solid #3a3c42',
      width: '100%',
    }}>
      <div style={{
        height: 48,
        background: 'linear-gradient(135deg, #be1c1c 0%, #8b1515 50%, #2b2d31 100%)',
      }} />
      <div style={{ padding: '0 14px 14px', display: 'flex', flexDirection: 'column', gap: 10 }}>
        <div style={{ marginTop: -22 }}>
          <img
            src={repakIcon}
            alt="Repak X"
            style={{
              width: 48,
              height: 48,
              borderRadius: 14,
              border: '3px solid #2b2d31',
              background: '#1e1f22',
              objectFit: 'cover',
            }}
          />
        </div>
        <div style={{ display: 'flex', flexDirection: 'column', gap: 4 }}>
          <div style={{ display: 'flex', alignItems: 'center', gap: 5, fontSize: '1rem', fontWeight: 700, color: '#f2f3f5' }}>
            <span>{serverName}</span>
            <CommunityBadge />
          </div>
          <div style={{ display: 'flex', alignItems: 'center', gap: 12, fontSize: '0.8rem', color: '#b5bac1' }}>
            <span style={{ display: 'flex', alignItems: 'center', gap: 5 }}>
              <span style={{ width: 8, height: 8, borderRadius: '50%', background: '#23a55a', flexShrink: 0 }} />
              {onlineCount} Online
            </span>
          </div>
        </div>
        <button
          onClick={() => openUrl(widgetData?.instant_invite || INVITE_URL)}
          style={{
            width: '100%',
            padding: 8,
            border: 'none',
            borderRadius: 6,
            background: '#248046',
            color: '#fff',
            fontSize: '0.85rem',
            fontWeight: 600,
            cursor: 'pointer',
            marginTop: 2,
          }}
          onMouseEnter={e => e.currentTarget.style.background = '#1a6334'}
          onMouseLeave={e => e.currentTarget.style.background = '#248046'}
        >
          Go to Server
        </button>
      </div>
    </div>
  );
}

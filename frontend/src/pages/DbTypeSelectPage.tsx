import { useTranslation } from 'react-i18next';
import './DbTypeSelectPage.css';

interface DbTypeSelectPageProps {
  onSelect: (dbType: string) => void;
  onCancel: () => void;
}

function DbTypeSelectPage({ onSelect, onCancel }: DbTypeSelectPageProps) {
  const { t } = useTranslation();

  return (
    <div className="dbtype-select-page">
      <div className="dbtype-select-card">
        <div className="card-header">
          <h2>{t('dbType.chooseTitle')}</h2>
          <p>{t('dbType.chooseSubtitle')}</p>
        </div>
        <div className="dbtype-options">
          <button
            className="dbtype-option"
            onClick={() => onSelect('postgres')}
          >
            <div className="dbtype-icon postgres-icon">
              <svg viewBox="0 0 64 64" width="48" height="48" fill="none">
                <ellipse cx="32" cy="32" rx="24" ry="26" fill="#336791" />
                <ellipse cx="32" cy="32" rx="20" ry="22" fill="#fff" opacity="0.15" />
                <text x="32" y="42" textAnchor="middle" fontFamily="serif" fontWeight="bold" fontSize="28" fill="#fff">P</text>
              </svg>
            </div>
            <span className="dbtype-label">PostgreSQL</span>
            <span className="dbtype-desc">{t('dbType.postgresDesc')}</span>
          </button>
          <button
            className="dbtype-option"
            onClick={() => onSelect('mysql')}
          >
            <div className="dbtype-icon mysql-icon">
              <svg viewBox="0 0 64 64" width="48" height="48" fill="none">
                <ellipse cx="32" cy="32" rx="24" ry="26" fill="#00758F" />
                <ellipse cx="32" cy="32" rx="20" ry="22" fill="#fff" opacity="0.15" />
                <text x="32" y="42" textAnchor="middle" fontFamily="serif" fontWeight="bold" fontSize="24" fill="#F29111">My</text>
              </svg>
            </div>
            <span className="dbtype-label">MySQL</span>
            <span className="dbtype-desc">{t('dbType.mysqlDesc')}</span>
          </button>
        </div>
        <div className="dbtype-actions">
          <button className="btn btn-ghost" onClick={onCancel}>
            {t('common.cancel')}
          </button>
        </div>
      </div>
    </div>
  );
}

export default DbTypeSelectPage;

import { Link } from "react-router-dom";
import styles from "./NotFoundPage.module.css";

export default function NotFoundPage() {
  return (
    <div className={styles.page}>
      <div className={styles.code}>404</div>
      <p>This page does not exist.</p>
      <Link to="/" className={styles.link}>
        Back to dashboard
      </Link>
    </div>
  );
}

import { getSessionUser } from "../../api/hooks/getAuthSessions";
import { Logo, Profile } from "./Profile";

const HeaderProfile = () => {
  return (
    <div className="flex flex-row justify-between">
      <Logo />
      <Profile />
    </div>
  );
};

export default HeaderProfile;

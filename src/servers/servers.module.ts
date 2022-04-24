import { Module } from '@nestjs/common';
import { ServersService } from './servers.service';
import { ServersResolver } from './servers.resolver';
import { PrismaService} from '../../prisma/prisma.service';

@Module({
  providers: [PrismaService, ServersResolver, ServersService]
})
export class ServersModule {}

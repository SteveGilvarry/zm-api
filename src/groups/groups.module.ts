import { Module } from '@nestjs/common';
import { GroupsService } from './groups.service';
import { GroupsResolver } from './groups.resolver';
import { PrismaService} from '../../prisma/prisma.service';

@Module({
  providers: [PrismaService,GroupsResolver, GroupsService]
})
export class GroupsModule {}

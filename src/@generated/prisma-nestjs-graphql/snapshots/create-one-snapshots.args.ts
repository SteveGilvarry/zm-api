import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { SnapshotsCreateInput } from './snapshots-create.input';
import { Type } from 'class-transformer';

@ArgsType()
export class CreateOneSnapshotsArgs {

    @Field(() => SnapshotsCreateInput, {nullable:false})
    @Type(() => SnapshotsCreateInput)
    data!: SnapshotsCreateInput;
}

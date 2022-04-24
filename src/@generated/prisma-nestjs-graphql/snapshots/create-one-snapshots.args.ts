import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { SnapshotsCreateInput } from './snapshots-create.input';

@ArgsType()
export class CreateOneSnapshotsArgs {

    @Field(() => SnapshotsCreateInput, {nullable:false})
    data!: SnapshotsCreateInput;
}

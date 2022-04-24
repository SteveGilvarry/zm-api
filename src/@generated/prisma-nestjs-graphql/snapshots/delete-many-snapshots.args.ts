import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { SnapshotsWhereInput } from './snapshots-where.input';

@ArgsType()
export class DeleteManySnapshotsArgs {

    @Field(() => SnapshotsWhereInput, {nullable:true})
    where?: SnapshotsWhereInput;
}

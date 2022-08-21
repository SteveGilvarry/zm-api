import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { SnapshotsWhereInput } from './snapshots-where.input';
import { Type } from 'class-transformer';

@ArgsType()
export class DeleteManySnapshotsArgs {

    @Field(() => SnapshotsWhereInput, {nullable:true})
    @Type(() => SnapshotsWhereInput)
    where?: SnapshotsWhereInput;
}

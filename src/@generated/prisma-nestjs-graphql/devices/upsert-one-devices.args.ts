import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { DevicesWhereUniqueInput } from './devices-where-unique.input';
import { Type } from 'class-transformer';
import { DevicesCreateInput } from './devices-create.input';
import { DevicesUpdateInput } from './devices-update.input';

@ArgsType()
export class UpsertOneDevicesArgs {

    @Field(() => DevicesWhereUniqueInput, {nullable:false})
    @Type(() => DevicesWhereUniqueInput)
    where!: DevicesWhereUniqueInput;

    @Field(() => DevicesCreateInput, {nullable:false})
    @Type(() => DevicesCreateInput)
    create!: DevicesCreateInput;

    @Field(() => DevicesUpdateInput, {nullable:false})
    @Type(() => DevicesUpdateInput)
    update!: DevicesUpdateInput;
}

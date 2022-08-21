import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { DevicesUpdateInput } from './devices-update.input';
import { Type } from 'class-transformer';
import { DevicesWhereUniqueInput } from './devices-where-unique.input';

@ArgsType()
export class UpdateOneDevicesArgs {

    @Field(() => DevicesUpdateInput, {nullable:false})
    @Type(() => DevicesUpdateInput)
    data!: DevicesUpdateInput;

    @Field(() => DevicesWhereUniqueInput, {nullable:false})
    @Type(() => DevicesWhereUniqueInput)
    where!: DevicesWhereUniqueInput;
}

import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { DevicesCreateInput } from './devices-create.input';
import { Type } from 'class-transformer';

@ArgsType()
export class CreateOneDevicesArgs {

    @Field(() => DevicesCreateInput, {nullable:false})
    @Type(() => DevicesCreateInput)
    data!: DevicesCreateInput;
}

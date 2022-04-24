import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { ZonesCreateInput } from './zones-create.input';

@ArgsType()
export class CreateOneZonesArgs {

    @Field(() => ZonesCreateInput, {nullable:false})
    data!: ZonesCreateInput;
}
